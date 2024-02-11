use serde::{Deserialize, Serialize};
use crypto::U32Token;

use crate::error::AssetIoError;
use crate::json::{skin::SkinFile, scene::SceneFile, model::ModelFile, icon::IconFile, animation::AnimFile, mesh::MeshFile, palette::PaletteFile, skeleton::SkelFile};

pub type AssetId = U32Token;

#[derive(Serialize, Deserialize, Clone)]
pub struct Asset {
    meta: AssetMeta,
    data: AssetData,
}

impl Asset {
    pub(crate) fn new(meta: AssetMeta, data: AssetData) -> Self {
        Self {
            meta,
            data,
        }
    }

    pub(crate) fn read(bytes: &[u8]) -> Result<Self, AssetIoError> {
        serde_json::from_slice(bytes).map_err(|e| AssetIoError::Message(e.to_string()))
    }

    pub(crate) fn deconstruct(self) -> (AssetMeta, AssetData) {
        (self.meta, self.data)
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AssetMeta {
    asset_id: String,
    schema_version: u32,
}

impl AssetMeta {
    pub fn new(asset_id: &AssetId, schema_version: u32) -> Self {
        Self {
            asset_id: asset_id.as_string(),
            schema_version,
        }
    }

    pub fn asset_id(&self) -> AssetId {
        AssetId::from_str(&self.asset_id).unwrap()
    }

    pub fn read_from_file(bytes: &[u8]) -> Result<Self, AssetIoError> {
        let (meta, _) = Asset::read(bytes)?.deconstruct();
        return Ok(meta);
    }
}

// Container
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum AssetData {
    Palette(PaletteFile),
    Skeleton(SkelFile),
    Mesh(MeshFile),
    Animation(AnimFile),
    Icon(IconFile),
    Skin(SkinFile),
    Scene(SceneFile),
    Model(ModelFile),
}