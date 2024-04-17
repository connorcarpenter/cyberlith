use std::{net::SocketAddr, pin::Pin};

use smol::future::Future;

use logging::{info, warn};

use http_common::{Request, Response, ResponseError};

use crate::Server;

// serves files from the file system
pub trait FileServer {
    fn serve_file(&mut self, path: &str, file_path: &str, file_name: &str);
}

impl FileServer for Server {
    fn serve_file(&mut self, url_path: &str, file_path: &str, file_name: &str) {
        let url_path = format!("GET /{}", url_path);

        info!("serving file @ {}", url_path);

        let file_path = format!("{}/{}", file_path, file_name);
        let new_endpoint = endpoint_2(&file_path);
        self.internal_insert_endpoint(url_path, new_endpoint);
    }
}

fn endpoint_2(
    file_path: &str,
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
    let file_path = file_path.to_string();
    Box::new(move |_args: (SocketAddr, Request)| {
        let file_path = file_path.clone();

        // convert typed future to pure future
        let pure_future = async move {
            let mut response = Response::default();

            // info!("reading file: {}", file_path);

            let Ok(bytes) = std::fs::read(&file_path) else {
                warn!("file not found: {}", &file_path);
                return Err(ResponseError::NotFound);
            };

            response.body = bytes;

            // info!("adding headers");

            // add Content-Type header
            let content_type = match file_path.split('.').last().unwrap() {
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

            // info!("sending response 1");

            return Ok(response);
        };

        Box::pin(pure_future)
    })
}
