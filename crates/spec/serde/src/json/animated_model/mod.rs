use std::collections::HashMap;

use cfg_if::cfg_if;
use serde::{Deserialize, Serialize};

use asset_id::AssetId;

cfg_if! {
    if #[cfg(feature = "read_json")] {
        mod read;
    } else {}
}

cfg_if! {
    if #[cfg(feature = "write_json")] {
        mod write;
    } else {}
}

// AnimatedModel

#[derive(Serialize, Deserialize, Clone)]
pub struct AnimatedModelJson {
    model_asset_id: String,
    // animation name -> animation asset id
    animations: HashMap<String, String>,
}

impl AnimatedModelJson {
    pub const CURRENT_SCHEMA_VERSION: u32 = 0;

    pub fn new() -> Self {
        Self {
            model_asset_id: String::new(),
            animations: HashMap::new(),
        }
    }

    pub fn dependencies(&self) -> Vec<AssetId> {
        let mut output = Vec::new();

        output.push(self.get_model_asset_id());

        for (_, asset_id) in self.animations.iter() {
            output.push(AssetId::from_str(asset_id).unwrap());
        }

        output
    }

    pub fn get_model_asset_id(&self) -> AssetId {
        AssetId::from_str(&self.model_asset_id).unwrap()
    }

    pub fn set_model_asset_id(&mut self, asset_id: &AssetId) {
        self.model_asset_id = asset_id.as_string();
    }

    pub fn get_animations(&self) -> &HashMap<String, String> {
        &self.animations
    }

    pub fn add_animation(&mut self, name: &str, asset_id: &AssetId) {
        self.animations
            .insert(name.to_string(), asset_id.as_string());
    }
}
