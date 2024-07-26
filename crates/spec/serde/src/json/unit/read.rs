use spec::Unit;

use crate::json::UnitJson;

impl Into<Unit> for UnitJson {
    fn into(self) -> Unit {
        Unit::new(
            self.get_animated_model_asset_id(),
            self.get_movement_config_asset_id(),
        )
    }
}
