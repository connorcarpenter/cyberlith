use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature = "read_json")] {
        mod read;
        pub use read::*;
    } else {}
}

cfg_if! {
    if #[cfg(feature = "write_json")] {
        mod write;
    } else {}
}

use asset_id::AssetId;
use serde::{Deserialize, Serialize};
use spec_serde::json::{AnimatedModelJson, MovementConfigJson, UnitJson};

use crate::json::{animation::AnimationJson, icon::IconJson, mesh::MeshJson, model::ModelJson, palette::PaletteJson, scene::SceneJson, skeleton::SkeletonJson, skin::SkinJson, UiConfigJson};

#[derive(Serialize, Deserialize, Clone)]
pub struct Asset {
    meta: AssetMeta,
    data: AssetData,
}

impl Asset {
    pub fn deconstruct(self) -> (AssetMeta, AssetData) {
        (self.meta, self.data)
    }

    pub fn meta(&self) -> &AssetMeta {
        &self.meta
    }

    pub fn data(&self) -> &AssetData {
        &self.data
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

    pub fn schema_version(&self) -> u32 {
        self.schema_version
    }
}

// Container
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum AssetData {
    Palette(PaletteJson),
    Skeleton(SkeletonJson),
    Mesh(MeshJson),
    Animation(AnimationJson),
    Icon(IconJson),
    Skin(SkinJson),
    Scene(SceneJson),
    Model(ModelJson),
    Ui(UiConfigJson),
    AnimatedModel(AnimatedModelJson),
    MovementConfig(MovementConfigJson),
    Unit(UnitJson),
}

impl AssetData {
    pub fn type_name(&self) -> String {
        match self {
            Self::Palette(_) => "palette",
            Self::Skeleton(_) => "skeleton",
            Self::Mesh(_) => "mesh",
            Self::Animation(_) => "animation",
            Self::Icon(_) => "icon",
            Self::Skin(_) => "skin",
            Self::Scene(_) => "scene",
            Self::Model(_) => "model",
            Self::Ui(_) => "ui",
            Self::AnimatedModel(_) => "animated_model",
            Self::MovementConfig(_) => "movement_config",
            Self::Unit(_) => "unit",
        }
        .to_string()
    }
}
