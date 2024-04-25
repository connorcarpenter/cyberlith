use std::{net::SocketAddr};

use http_client::{ResponseError};
use http_server::{async_dup::Arc, Request, smol::{io::AsyncWriteExt, lock::RwLock}, Response};

use crate::file_metadata_store::{FileMetadata, FileType};
use crate::state::State;

pub(crate) async fn file_endpoint_handler(
    _addr: SocketAddr,
    _incoming_request: Request,
    state: Arc<RwLock<State>>,
    file_name: String,
) -> Result<Response, ResponseError> {

    let metadata: FileMetadata = {
        let mut state_guard = state.read().await;

        let Some(metadata) = state_guard.get_metadata(&file_name) else {
            return Err(ResponseError::NotFound);
        };

        metadata.clone()
    };

    let path = metadata.path().to_string();
    let mut state_guard = state.write().await;

    let Some(file_bytes) = state_guard.cache_load(&path) else {

        return Err(ResponseError::InternalServerError(format!(
            "Failed to load file data for path: `{}`",
            path
        )));
    };

    let file_type = metadata.file_type();

    let mut response = Response::default();

    response.body = file_bytes;

    //// Headers

    // add Content-Type header

    let content_type = match file_type {
        FileType::Html => "text/html",
        FileType::Js => "application/javascript",
        FileType::Wasm => "application/wasm",
    };
    response
        .headers
        .insert("Content-Type".to_string(), content_type.to_string());

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