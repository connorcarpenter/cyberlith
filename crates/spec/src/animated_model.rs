use std::collections::HashMap;

use asset_id::AssetId;

pub struct AnimatedModel {
    model_asset_id: AssetId,
    animations: HashMap<String, AssetId>,
}

impl AnimatedModel {
    pub fn new(model_asset_id: AssetId) -> Self {
        Self {
            model_asset_id,
            animations: HashMap::new(),
        }
    }

    pub fn get_model_asset_id(&self) -> AssetId {
        self.model_asset_id
    }

    pub fn get_animations(&self) -> &HashMap<String, AssetId> {
        &self.animations
    }

    pub fn add_animation(&mut self, name: &str, asset_id: AssetId) {
        self.animations.insert(name.to_string(), asset_id);
    }
}
