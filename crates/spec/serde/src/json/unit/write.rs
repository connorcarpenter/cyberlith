
use spec::Unit;

use crate::json::UnitJson;

impl From<&Unit> for UnitJson {
    fn from(value: &Unit) -> Self {
        let mut me = Self::new();
        me.set_animated_model_asset_id(&value.get_animated_model_asset_id());
        me.set_movement_config_asset_id(&value.get_movement_config_asset_id());
        me
    }
}