use crate::{
    error::AssetIoError,
    json::{AnimationJson, Asset, AssetData, AssetMeta},
};

impl AnimationJson {
    pub fn read(bytes: &[u8]) -> Result<(AssetMeta, Self), AssetIoError> {
        let (meta, data) = Asset::read(bytes)?.deconstruct();
        let AssetData::Animation(data) = data else {
            return Err(AssetIoError::Message("Invalid Asset Type".to_string()));
        };
        return Ok((meta, data));
    }
}
