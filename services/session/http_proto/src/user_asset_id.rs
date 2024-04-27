use naia_serde::SerdeInternal as Serde;

use bevy_http_shared::{ApiRequest, ApiResponse, Method};

use asset_id::AssetId;

// Request
#[derive(Serde, PartialEq, Clone)]
pub struct UserAssetIdRequest {
    world_instance_secret: String,
    user_id: u64,
    asset_id: AssetId,
    added: bool,
}

impl UserAssetIdRequest {
    pub fn new(world_instance_secret: &str, user_id: u64, asset_id: AssetId, added: bool) -> Self {
        Self {
            world_instance_secret: world_instance_secret.to_string(),
            user_id,
            asset_id,
            added,
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

    pub fn added(&self) -> bool {
        self.added
    }
}

// Response
#[derive(Serde, PartialEq, Clone)]
pub struct UserAssetIdResponse;

// Traits
impl ApiRequest for UserAssetIdRequest {
    type Response = UserAssetIdResponse;

    fn name() -> &'static str { "UserAssetIdRequest" }

    fn method() -> Method {
        Method::Post
    }

    fn path() -> &'static str {
        "user_asset_id"
    }
}

impl ApiResponse for UserAssetIdResponse {
    fn name() -> &'static str { "UserAssetIdResponse" }
}
