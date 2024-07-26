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
pub struct UnitBits {
    animated_model_asset_id: AssetId,
    movement_config_asset_id: AssetId,
}

impl UnitBits {
    pub fn new(animated_model_asset_id: AssetId, movement_config_asset_id: AssetId) -> Self {
        Self {
            animated_model_asset_id,
            movement_config_asset_id,
        }
    }

    pub fn get_animated_model_asset_id(&self) -> AssetId {
        self.animated_model_asset_id
    }

    pub fn get_movement_config_asset_id(&self) -> AssetId {
        self.movement_config_asset_id
    }
}
