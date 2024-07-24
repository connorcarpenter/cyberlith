
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

// MovementConfig

#[derive(Serialize, Deserialize, Clone)]
pub struct MovementConfigJson {
    max_velocity: f32, // in meters per second
}

impl MovementConfigJson {
    pub const CURRENT_SCHEMA_VERSION: u32 = 0;

    pub fn new() -> Self {
        Self {
            max_velocity: 0.0
        }
    }

    pub fn dependencies(&self) -> Vec<AssetId> {
        Vec::new()
    }

    pub fn get_max_velocity(&self) -> f32 {
        self.max_velocity
    }

    pub fn set_max_velocity(&mut self, val: f32) {
        self.max_velocity = val;
    }
}
