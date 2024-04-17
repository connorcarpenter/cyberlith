use std::{
    net::SocketAddr,
    pin::Pin,
};

use smol::future::Future;
use http_client_shared::fetch_async;

use logging::info;

use http_common::{Method, Request, Response, ResponseError};

use crate::{log_util, Server};

// serves files from a remote file server
pub trait RemoteFileServer {
    fn serve_remote_file(
        &mut self,
        host_name: &str, url_path: &str,
        remote_name: &str, remote_addr: &str, remote_port: &str, file_name: &str
    );
}

impl RemoteFileServer for Server {

    fn serve_remote_file(&mut self, host_name: &str, url_path: &str, remote_name: &str, remote_addr: &str, remote_port: &str, file_name: &str) {
        let url_path = format!("GET /{}", url_path);

        info!("serving remote file @ {}", url_path);

        let remote_url = format!("http://{}:{}/{}", remote_addr, remote_port, file_name);
        let new_endpoint = endpoint_2(host_name, remote_name, &remote_url);
        self.internal_insert_endpoint(url_path, new_endpoint);
    }
}

fn endpoint_2(
    host_name: &str,
    remote_name: &str,
    remote_url: &str,
) -> Box<
    dyn 'static
    + Send
    + Sync
    + FnMut(
        (SocketAddr, Request),
    ) -> Pin<
        Box<dyn 'static + Send + Sync + Future<Output = Result<Response, ResponseError>>>,
    >,
> {
    let host_name = host_name.to_string();
    let remote_name = remote_name.to_string();
    let remote_url = remote_url.to_string();
    Box::new(move |_args: (SocketAddr, Request)| {
        let host_name = host_name.clone();
        let remote_name = remote_name.clone();
        let remote_url = remote_url.clone();

        // convert typed future to pure future
        let pure_future = async move {
            let host_name = host_name.clone();
            let remote_name = remote_name.clone();

            // info!("fetching remote file");
            log_util::recv_req(&host_name, "client", &remote_url);

            let remote_req = Request::new(Method::Get, &remote_url, Vec::new());

            log_util::send_req(&host_name, &remote_name, &remote_url);
            let remote_response_result = fetch_async(remote_req).await;
            log_util::recv_res(&host_name, &remote_name, &remote_url);

            let mut response = Response::default();
            match remote_response_result {
                Ok(remote_response) => {
                    response.body = remote_response.body;
                },
                Err(err) => {
                    return Err(ResponseError::HttpError(format!("received error from remote server: {}", err.to_string())));
                }
            }

            // info!("adding headers");

            // add Content-Type header
            let content_type = match remote_url.split('.').last().unwrap() {
                "html" => "text/html",
                "js" => "application/javascript",
                "wasm" => "application/wasm",
                _ => "text/plain",
            };
            response
                .headers
                .insert("Content-Type".to_string(), content_type.to_string());

            // add Content-Length header
            response.headers.insert(
                "Content-Length".to_string(),
                response.body.len().to_string(),
            );

            log_util::send_res(&host_name, "client", &remote_url);

            return Ok(response);
        };

        Box::pin(pure_future)
    })
}
