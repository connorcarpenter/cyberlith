
use crate::{file_cache::FileCache, file_metadata_store::FileMetadataStore};

pub struct State {
    file_metadata_store: FileMetadataStore,
    file_cache: FileCache,
}

impl State {
    pub fn new(
        file_cache_size_kb: u32,
        file_metadata_store: FileMetadataStore,
    ) -> Self {
        Self {
            file_metadata_store,
            file_cache: FileCache::new(file_cache_size_kb),
        }
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
}
