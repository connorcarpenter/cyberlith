use serde::{Deserialize, Serialize};
use crypto::U32Token;

use crate::json::{skin::SkinFile, scene::SceneFile, model::ModelFile, icon::IconFile, animation::AnimFile, mesh::MeshFile, palette::PaletteFile, skeleton::SkelFile};

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

    pub fn write(&self) -> Vec<u8> {
        self.to_pretty_json()
    }

    pub fn to_pretty_json(&self) -> Vec<u8> {
        let mut out_bytes = Vec::new();
        serde_json::to_writer_pretty(&mut out_bytes, self).unwrap();
        out_bytes
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AssetMeta {
    asset_id: String,
    schema_version: u32,
}

impl AssetMeta {
    pub fn new(asset_id: &U32Token, schema_version: u32) -> Self {
        Self {
            asset_id: asset_id.as_string(),
            schema_version,
        }
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