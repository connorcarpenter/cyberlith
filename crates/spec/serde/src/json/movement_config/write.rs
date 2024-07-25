
use spec::MovementConfig;

use crate::json::MovementConfigJson;

impl From<&MovementConfig> for MovementConfigJson {
    fn from(value: &MovementConfig) -> Self {
        let mut me = Self::new();

        me.set_max_velocity(value.max_velocity());

        me
    }
}