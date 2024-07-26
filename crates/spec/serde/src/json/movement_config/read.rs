use spec::MovementConfig;

use crate::json::MovementConfigJson;

impl Into<MovementConfig> for MovementConfigJson {
    fn into(self) -> MovementConfig {
        MovementConfig::new(self.max_velocity)
    }
}
