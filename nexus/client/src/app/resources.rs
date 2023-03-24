use std::default::Default;

use bevy_ecs::prelude::Resource;

#[derive(Resource)]
pub struct Global;

impl Default for Global {
    fn default() -> Self {
        Self
    }
}
