
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

// Unit

#[derive(Serialize, Deserialize, Clone)]
pub struct UnitJson {
    animated_model_asset_id: String,
    movement_config_asset_id: String,
}

impl UnitJson {
    pub const CURRENT_SCHEMA_VERSION: u32 = 0;

    pub fn new() -> Self {
        Self {
            animated_model_asset_id: String::new(),
            movement_config_asset_id: String::new(),
        }
    }

    pub fn dependencies(&self) -> Vec<AssetId> {
        let mut output = Vec::new();

        output.push(self.get_animated_model_asset_id());
        output.push(self.get_movement_config_asset_id());

        output
    }

    pub fn get_animated_model_asset_id(&self) -> AssetId {
        AssetId::from_str(&self.animated_model_asset_id).unwrap()
    }

    pub fn set_animated_model_asset_id(&mut self, asset_id: &AssetId) {
        self.animated_model_asset_id = asset_id.as_string();
    }

    pub fn get_movement_config_asset_id(&self) -> AssetId {
        AssetId::from_str(&self.movement_config_asset_id).unwrap()
    }

    pub fn set_movement_config_asset_id(&mut self, asset_id: &AssetId) {
        self.movement_config_asset_id = asset_id.as_string();
    }
}
