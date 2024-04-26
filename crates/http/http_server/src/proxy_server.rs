use std::{net::SocketAddr, pin::Pin};

use smol::future::Future;

use http_client_shared::fetch_async;
use http_common::{ApiRequest, Method, Request, Response, ResponseError};
use logging::info;

use crate::{base_server::Endpoint, log_util, Server};

// serves a pass-through proxy
pub trait ProxyServer {
    fn serve_proxy(
        &mut self,
        host_name: &str,
        incoming_host: Option<(&str, Option<&str>)>,
        allow_origin_opt: Option<&str>,
        method: Method,
        url_path: &str,
        remote_name: &str,
        remote_addr: &str,
        remote_port: &str,
        file_name: &str,
    );
    fn serve_api_proxy<TypeRequest: 'static + ApiRequest>(
        &mut self,
        host_name: &str,
        incoming_host: Option<(&str, Option<&str>)>,
        allow_origin_opt: Option<&str>,
        remote_name: &str,
        remote_addr: &str,
        remote_port: &str,
    );
}

impl ProxyServer for Server {
    fn serve_proxy(
        &mut self,
        host_name: &str,
        incoming_host: Option<(&str, Option<&str>)>,
        allow_origin_opt: Option<&str>,
        incoming_method: Method,
        incoming_path: &str,
        remote_name: &str,
        remote_addr: &str,
        remote_port: &str,
        remote_path: &str,
    ) {
        let url_path = format!("{} /{}", incoming_method.as_str(), incoming_path);

        info!("serving proxy @ {}", url_path);

        let remote_url = format!("http://{}:{}/{}", remote_addr, remote_port, remote_path);
        let logged_remote_url = format!(
            "{} host:{}/{}",
            incoming_method.as_str(),
            remote_port,
            remote_path
        );
        let endpoint_func = get_endpoint_func(
            host_name,
            remote_name,
            allow_origin_opt,
            incoming_method,
            &remote_url,
            &logged_remote_url,
        );
        let incoming_host =
            incoming_host.map(|(rq, rdopt)| (rq.to_string(), rdopt.map(|rd| rd.to_string())));
        let new_endpoint = Endpoint::new(endpoint_func, incoming_host);
        self.internal_insert_endpoint(url_path, new_endpoint);
    }

    fn serve_api_proxy<TypeRequest: 'static + ApiRequest>(
        &mut self,
        host_name: &str,
        incoming_host: Option<(&str, Option<&str>)>,
        allow_origin_opt: Option<&str>,
        remote_name: &str,
        remote_addr: &str,
        remote_port: &str,
    ) {
        Self::serve_proxy(
            self,
            host_name,
            incoming_host,
            allow_origin_opt,
            TypeRequest::method(),
            TypeRequest::path(),
            remote_name,
            remote_addr,
            remote_port,
            TypeRequest::path(),
        );
    }
}

fn get_endpoint_func(
    host_name: &str,
    remote_name: &str,
    allow_origin_opt: Option<&str>,
    method: Method,
    remote_url: &str,
    logged_remote_url: &str,
) -> Box<
    dyn 'static
        + Send
        + Sync
        + Fn(
            (SocketAddr, Request),
        )
            -> Pin<Box<dyn 'static + Send + Sync + Future<Output = Result<Response, ResponseError>>>>,
> {
    let host_name = host_name.to_string();
    let remote_name = remote_name.to_string();
    let allow_origin_opt = allow_origin_opt.map(|s| s.to_string());
    let method = method.clone();
    let remote_url = remote_url.to_string();
    let logged_remote_url = logged_remote_url.to_string();
    Box::new(move |args: (SocketAddr, Request)| {
        let outer_req = args.1;
        let outer_url = outer_req.url;
        let outer_headers = outer_req.headers;
        let outer_body = outer_req.body;

        let host_name = host_name.clone();
        let remote_name = remote_name.clone();
        let allow_origin_opt = allow_origin_opt.clone();
        let method = method.clone();
        let remote_url = remote_url.clone();
        let logged_remote_url = logged_remote_url.clone();

        // convert typed future to pure future
        let pure_future = async move {
            let host_name = host_name.clone();
            let remote_name = remote_name.clone();
            let logged_remote_url = logged_remote_url.clone();
            let logged_host_url = format!("{} {}", method.as_str(), outer_url);

            logging::info!("[");
            log_util::recv_req(&host_name, &logged_host_url);

            let mut remote_req = Request::new(method, &remote_url, outer_body);
            remote_req.headers = outer_headers;

            log_util::send_req(&host_name, &remote_name, &logged_remote_url);
            let remote_response_result = fetch_async(remote_req).await;
            log_util::recv_res(&host_name, &remote_name, &logged_remote_url);

            let mut response = Response::default();
            match remote_response_result {
                Ok(remote_response) => {
                    response.url = outer_url;
                    response.ok = remote_response.ok;
                    response.status = remote_response.status;
                    response.status_text = remote_response.status_text;
                    response.body = remote_response.body;

                    // for (header_name, _) in remote_response.headers.iter() {
                    //     info!("incoming req has header: {}", header_name);
                    // }

                    // pass through headers
                    for header_name in [
                        "content-type",
                        "content-length",
                        "content-encoding",
                        "etag",
                        "cache-control",
                        "access-control-allow-headers",
                    ] {
                        if remote_response.headers.contains_key(header_name) {
                            // info!("adding header: {}", header_name);
                            let remote_header_value =
                                remote_response.headers.get(header_name).unwrap();
                            response
                                .headers
                                .insert(header_name.to_string(), remote_header_value.clone());
                        } else {
                            // info!("header not found: {}", header_name);
                        }
                    }

                    // access control allow origin
                    if let Some(allow_origin) = allow_origin_opt {
                        while response.headers.contains_key("access-control-allow-origin") {
                            response.headers.remove("access-control-allow-origin");
                        }
                        response
                            .headers
                            .insert("access-control-allow-origin".to_string(), allow_origin);
                    }
                }
                Err(err) => {
                    return Err(ResponseError::HttpError(format!(
                        "received error from remote server: {}",
                        err.to_string()
                    )));
                }
            }

            log_util::send_res(&host_name, &logged_host_url);
            logging::info!("]");

            return Ok(response);
        };

        Box::pin(pure_future)
    })
}
