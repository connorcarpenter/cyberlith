use serde::{Deserialize, Serialize};

use crate::json::{skin::SkinFile, scene::SceneFile, model::ModelFile, icon::IconFile, animation::AnimFile, mesh::MeshFile, palette::PaletteFile, skeleton::SkelFile};

#[derive(Serialize, Deserialize, Clone)]
pub struct Asset {
    pub meta: AssetMeta,
    pub data: AssetData,
}

impl Asset {
    pub fn to_pretty_json(&self) -> Vec<u8> {
        let mut out_bytes = Vec::new();
        serde_json::to_writer_pretty(&mut out_bytes, self).unwrap();
        out_bytes
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AssetMeta {
    pub asset_id: String,
    pub schema_version: u32,
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