use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature = "read_bits")] {
        mod read;
    } else {}
}

cfg_if! {
    if #[cfg(feature = "write_bits")] {
        mod write;
    } else {}
}

use naia_serde::SerdeInternal as Serde;

use asset_id::AssetId;

#[derive(Serde, Clone, PartialEq)]
pub struct AnimatedModelBits {
    model_asset_id: AssetId,
    animations: Vec<(String, AssetId)>,
}

impl AnimatedModelBits {
    pub fn new(model_asset_id: AssetId) -> Self {
        Self {
            model_asset_id,
            animations: Vec::new(),
        }
    }

    pub fn get_model_asset_id(&self) -> AssetId {
        self.model_asset_id
    }

    pub fn add_animation(&mut self, name: &str, asset_id: AssetId) {
        self.animations.push((name.to_string(), asset_id));
    }

    pub fn get_animations(&self) -> &Vec<(String, AssetId)> {
        &self.animations
    }
}
