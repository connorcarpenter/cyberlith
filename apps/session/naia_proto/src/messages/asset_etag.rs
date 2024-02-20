use naia_bevy_shared::{Message, Request, Response, Serde};

use asset_id::{AssetId, ETag};

#[derive(Message, Debug)]
pub struct AssetEtagRequest {
    pub asset_id: AssetId,
}

impl Request for AssetEtagRequest {
    type Response = AssetEtagResponse;
}

impl AssetEtagRequest {
    pub fn new(asset_id: &AssetId) -> Self {
        Self { asset_id: *asset_id }
    }
}

#[derive(Serde, Clone, Eq, PartialEq, Hash, Debug)]
pub enum AssetEtagResponseValue {
    Found(ETag),
    NotFound,
}

#[derive(Message, Eq, PartialEq, Hash, Debug)]
pub struct AssetEtagResponse {
    pub value: AssetEtagResponseValue
}

impl Response for AssetEtagResponse {

}

impl AssetEtagResponse {
    pub fn new(value: AssetEtagResponseValue) -> Self {
        Self { value }
    }
}