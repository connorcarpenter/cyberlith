use asset_id::AssetId;

pub struct Unit {
    animated_model_asset_id: AssetId,
    movement_config_asset_id: AssetId,
}

impl Unit {
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
