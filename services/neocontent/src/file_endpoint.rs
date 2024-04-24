use std::{net::SocketAddr};

use http_client::{ResponseError};
use http_server::{async_dup::Arc, Request, smol::{io::AsyncWriteExt, lock::RwLock}, Response};

use crate::file_metadata_store::FileType;
use crate::state::State;

pub(crate) async fn file_endpoint_handler(
    _addr: SocketAddr,
    _incoming_request: Request,
    state: Arc<RwLock<State>>,
    file_name: String,
) -> Result<Response, ResponseError> {

    // let req_etag_opt = request.etag_opt();
    let state_guard = state.read().await;

    let Some(metadata) = state_guard.get_metadata(&file_name) else {
        return Err(ResponseError::NotFound);
    };

    // let file_etag = metadata.etag();
    // if let Some(req_etag) = req_etag_opt {
    //     if file_etag == req_etag {
    //         return Ok(AssetResponse::not_modified());
    //     }
    // }

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

    // info!("adding headers");

    // add Content-Type header
    let content_type = match file_type {
        FileType::Html => "text/html",
        FileType::Js => "application/javascript",
        FileType::Wasm => "application/wasm",
        // _ => panic!("unsupported file type: {:?}", file_type),
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
}

//
// pub fn handle_asset_request(
//     &mut self,
//     request: AssetRequest,
// ) -> Result<AssetResponse, ResponseError> {
//     let req_asset_id = request.asset_id();
//     let req_etag_opt = request.etag_opt();
//
//     if let Some(metadata) = self.asset_metadata_store.get(&req_asset_id) {
//         let asset_etag = metadata.etag();
//         if let Some(req_etag) = req_etag_opt {
//             if asset_etag == req_etag {
//                 return Ok(AssetResponse::not_modified());
//             }
//         }
//
//         let path = metadata.path().to_string();
//         let Some(asset_data) = self.asset_cache.load(&path) else {
//             return Err(ResponseError::InternalServerError(format!(
//                 "Failed to load asset data for path: `{}`",
//                 path
//             )));
//         };
//
//         let asset_type = metadata.asset_type();
//
//         let dependencies = metadata.dependencies().clone();
//         return Ok(AssetResponse::modified(
//             asset_etag,
//             asset_type,
//             dependencies,
//             asset_data,
//         ));
//     } else {
//         return Err(ResponseError::NotFound);
//     }
// }