use naia_serde::SerdeInternal as Serde;

use bevy_http_shared::{ApiRequest, ApiResponse, Method};

use asset_id::AssetId;

// Request
#[derive(Serde, PartialEq, Clone)]
pub struct AddedAssetIdRequest {
    world_instance_secret: String,
    user_id: u64,
    asset_id: AssetId,
}

impl AddedAssetIdRequest {
    pub fn new(world_instance_secret: &str, user_id: u64, asset_id: AssetId) -> Self {
        Self {
            world_instance_secret: world_instance_secret.to_string(),
            user_id,
            asset_id,
        }
    }

    pub fn world_instance_secret(&self) -> &str {
        &self.world_instance_secret
    }

    pub fn user_id(&self) -> u64 {
        self.user_id
    }

    pub fn asset_id(&self) -> &AssetId {
        &self.asset_id
    }
}

// Response
#[derive(Serde, PartialEq, Clone)]
pub struct AddedAssetIdResponse;

// Traits
impl ApiRequest for AddedAssetIdRequest {
    type Response = AddedAssetIdResponse;

    fn method() -> Method {
        Method::Post
    }

    fn path() -> &'static str {
        "added_asset_id"
    }
}

impl ApiResponse for AddedAssetIdResponse {}
