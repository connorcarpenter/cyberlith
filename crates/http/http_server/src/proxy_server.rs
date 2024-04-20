use std::{net::SocketAddr, pin::Pin};

use smol::future::Future;

use http_client_shared::fetch_async;
use http_common::{ApiRequest, Method, Request, Response, ResponseError};
use logging::info;

use crate::{log_util, Server, base_server::Endpoint};

// serves a pass-through proxy
pub trait ProxyServer {
    fn serve_proxy(
        &mut self,
        host_name: &str,
        incoming_host: Option<(&str, Option<&str>)>,
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
        let logged_remote_url = format!("{} host:{}/{}", incoming_method.as_str(), remote_port, remote_path);
        let endpoint_func = get_endpoint_func(host_name, remote_name, incoming_method, &remote_url, &logged_remote_url);
        let incoming_host = incoming_host.map(|(rq, rdopt)| (rq.to_string(), rdopt.map(|rd| rd.to_string())));
        let new_endpoint = Endpoint::new(endpoint_func, incoming_host);
        self.internal_insert_endpoint(url_path, new_endpoint);
    }

    fn serve_api_proxy<TypeRequest: 'static + ApiRequest>(
        &mut self,
        host_name: &str,
        incoming_host: Option<(&str, Option<&str>)>,
        remote_name: &str,
        remote_addr: &str,
        remote_port: &str,
    ) {
        Self::serve_proxy(
            self,
            host_name,
            incoming_host,
            TypeRequest::method(),
            TypeRequest::path(),
            remote_name,
            remote_addr,
            remote_port,
            TypeRequest::path()
        );
    }
}

fn get_endpoint_func(
    host_name: &str,
    remote_name: &str,
    method: Method,
    remote_url: &str,
    logged_remote_url: &str,
) -> Box<
    dyn 'static
        + Send
        + Sync
        + Fn(
            (SocketAddr, Request),
        ) -> Pin<
            Box<dyn 'static + Send + Sync + Future<Output = Result<Response, ResponseError>>>,
        >,
> {
    let host_name = host_name.to_string();
    let remote_name = remote_name.to_string();
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
                    response.body = remote_response.body;
                }
                Err(err) => {
                    return Err(ResponseError::HttpError(format!(
                        "received error from remote server: {}",
                        err.to_string()
                    )));
                }
            }

            // info!("adding headers");

            // add Content-Type header
            let content_type = match remote_url.split('.').last().unwrap() {
                "html" => Some("text/html"),
                "js" => Some("application/javascript"),
                "wasm" => Some("application/wasm"),
                "txt" => Some("text/plain"),
                _ => None,
            };
            if let Some(content_type) = content_type {
                response
                    .headers
                    .insert("Content-Type".to_string(), content_type.to_string());
            }

            // add Content-Length header
            response.headers.insert(
                "Content-Length".to_string(),
                response.body.len().to_string(),
            );

            log_util::send_res(&host_name, &logged_host_url);
            logging::info!("]");

            return Ok(response);
        };

        Box::pin(pure_future)
    })
}
