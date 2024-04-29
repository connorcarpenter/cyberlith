use std::net::SocketAddr;

use http_client_shared::fetch_async;
use http_common::{ApiRequest, Method, Request, Response};
use logging::{info};

use crate::{log_util, Server, endpoint::{EndpointRef, Endpoint, EndpointFunc}};

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
    ) -> EndpointRef;
    fn serve_api_proxy<TypeRequest: 'static + ApiRequest>(
        &mut self,
        host_name: &str,
        incoming_host: Option<(&str, Option<&str>)>,
        allow_origin_opt: Option<&str>,
        remote_name: &str,
        remote_addr: &str,
        remote_port: &str,
    ) -> EndpointRef;
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
    ) -> EndpointRef {
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
        self.internal_insert_endpoint(url_path.clone(), new_endpoint);
        EndpointRef::new(self, url_path)
    }

    fn serve_api_proxy<TypeRequest: 'static + ApiRequest>(
        &mut self,
        host_name: &str,
        incoming_host: Option<(&str, Option<&str>)>,
        allow_origin_opt: Option<&str>,
        remote_name: &str,
        remote_addr: &str,
        remote_port: &str,
    ) -> EndpointRef {
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
        )
    }
}

fn get_endpoint_func(
    host_name: &str,
    remote_name: &str,
    allow_origin_opt: Option<&str>,
    method: Method,
    remote_url: &str,
    logged_remote_url: &str,
) -> EndpointFunc {
    let host_name = host_name.to_string();
    let remote_name = remote_name.to_string();
    let allow_origin_opt = allow_origin_opt.map(|s| s.to_string());
    let method = method.clone();
    let remote_url = remote_url.to_string();
    let logged_remote_url = logged_remote_url.to_string();
    Box::new(move |_outer_addr: SocketAddr, outer_req: Request| {
        let outer_headers: Vec<(String, String)> = outer_req.headers_iter().map(|(name, value)| (name.clone(), value.clone())).collect();
        let outer_url = outer_req.url;
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
            log_util::recv_req(&host_name, &logged_host_url, "");

            let mut remote_req = Request::new(method, &remote_url, outer_body);
            for (header_name, header_value) in outer_headers.iter() {
                remote_req.set_header(header_name, header_value);
            }

            log_util::send_req(&host_name, &remote_name, &logged_remote_url);
            let remote_response_result = fetch_async(remote_req).await;
            log_util::recv_res(&host_name, &remote_name, &logged_remote_url);

            let mut response = Response::default();
            match remote_response_result {
                Ok(remote_response) => {

                    // pass through headers
                    for header_name in [
                        "content-type",
                        "content-length",
                        "content-encoding",
                        "etag",
                        "cache-control",
                        "access-control-allow-headers",
                        "set-cookie",
                    ] {
                        if remote_response.has_header(header_name) {
                            // info!("adding header: {}", header_name);
                            let remote_header_value =
                                remote_response.get_header(header_name).unwrap();
                            response.set_header(header_name, remote_header_value);
                        } else {
                            // info!("header not found: {}", header_name);
                        }
                    }

                    response.url = outer_url;
                    response.ok = remote_response.ok;
                    response.status = remote_response.status;
                    response.status_text = remote_response.status_text;
                    response.body = remote_response.body;

                    // access control allow origin
                    if let Some(allow_origin) = allow_origin_opt {
                        while response.has_header("access-control-allow-origin") {
                            response.remove_header("access-control-allow-origin");
                        }
                        response
                            .set_header("access-control-allow-origin", &allow_origin);
                    }
                }
                Err(err) => {
                    response = err.to_response(&outer_url);
                }
            }

            log_util::send_res(&host_name, format!("{} {}", response.status, response.status_text).as_str());
            logging::info!("]");

            return Ok(response);
        };

        Box::pin(pure_future)
    })
}
