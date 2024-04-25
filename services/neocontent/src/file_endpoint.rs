use std::{net::SocketAddr};

use asset_id::ETag;
use http_client::{ResponseError};
use http_server::{async_dup::Arc, Request, smol::{lock::RwLock}, Response};
use logging::info;

use crate::{state::State, file_metadata_store::{FileMetadata, FileType}};

pub(crate) async fn file_endpoint_handler(
    _addr: SocketAddr,
    incoming_request: Request,
    state: Arc<RwLock<State>>,
    file_name: String,
) -> Result<Response, ResponseError> {

    let metadata: FileMetadata = {
        let state_guard = state.read().await;

        let Some(metadata) = state_guard.get_metadata(&file_name) else {
            return Err(ResponseError::NotFound);
        };

        metadata.clone()
    };

    if incoming_request.headers.contains_key("If-None-Match") {
        let incoming_etag_str = incoming_request.headers.get("If-None-Match").unwrap();
        if let Ok(incoming_etag) = ETag::from_str(incoming_etag_str) {
            if incoming_etag == metadata.etag() {
                info!("Incoming request matched ETag: {}, returning 304 Not Modified response", incoming_etag_str);

                let mut response = Response::not_modified(&incoming_request.url);

                add_caching_headers(metadata, &mut response);

                response.headers.insert(
                    "Content-Length".to_string(),
                    response.body.len().to_string(),
                );

                return Ok(response);
            }
        }
    }

    let file_bytes = {
        let mut state_guard = state.write().await;

        let Some(file_bytes) = state_guard.cache_load(metadata.path()) else {
            return Err(ResponseError::InternalServerError(format!(
                "Failed to load file data for path: `{}`",
                metadata.path()
            )));
        };

        file_bytes
    };

    let file_type = metadata.file_type();

    let mut response = Response::default();

    response.body = file_bytes;

    //// Headers

    add_caching_headers(metadata, &mut response);

    // add Content-Encoding header

    // the content server files are ALWAY brotli-compressed!
    response.headers.insert("Content-Encoding".to_string(), "br".to_string());

    // add Content-Length header

    response.headers.insert(
        "Content-Length".to_string(),
        response.body.len().to_string(),
    );

    return Ok(response);
}

fn add_caching_headers(metadata: FileMetadata, response: &mut Response) {
    // add Content-Type header
    let content_type = match metadata.file_type() {
        FileType::Html => "text/html",
        FileType::Js => "application/javascript",
        FileType::Wasm => "application/wasm",
    };
    response
        .headers
        .insert("Content-Type".to_string(), content_type.to_string());

    // add ETag header
    response.headers.insert("ETag".to_string(), metadata.etag().to_string());

    // add cache-control header
    response.headers.insert("Cache-Control".to_string(), "public, no-cache, max-age=0".to_string());
}