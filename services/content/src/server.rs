use std::{
    net::SocketAddr,
    pin::Pin,
};

use logging::{info, warn};
use smol::future::Future;
use config::CONTENT_SERVER_FILES_PATH;
use http_common::{Request, Response, ResponseError};
use http_server::Server;

pub(crate) trait FileServer {
    fn serve_file(&mut self, file_name: &str);
    fn serve_file_masked(&mut self, path: &str, file_name: &str);
}

impl FileServer for Server {

    fn serve_file(&mut self, file_name: &str) {
        self.serve_file_masked(file_name, file_name);
    }

    fn serve_file_masked(&mut self, path: &str, file_name: &str) {
        let endpoint_path = format!("GET /{}", path);

        info!("will serve file at: {}", endpoint_path);
        let new_endpoint = endpoint_2(file_name);
        self.internal_insert_endpoint(endpoint_path, new_endpoint);
    }
}

fn endpoint_2(
    file_name: &str,
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
    let file_name = file_name.to_string();
    Box::new(move |args: (SocketAddr, Request)| {
        let _addr = args.0;
        let _pure_request = args.1;
        let file_name = file_name.clone();

        // convert typed future to pure future
        let pure_future = async move {
            let mut response = Response::default();

            // info!("reading file: {}", file_name);

            let file_path = format!("{}{}", CONTENT_SERVER_FILES_PATH, file_name);
            let Ok(bytes) = std::fs::read(&file_path) else {
                warn!("file not found: {}", &file_path);
                return Err(ResponseError::NotFound);
            };

            response.body = bytes;

            // info!("adding headers");

            // add Content-Type header
            let content_type = match file_name.split('.').last().unwrap() {
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
