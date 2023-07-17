use std::default::Default;

use bevy_ecs::component::Component;

#[derive(Component)]
pub struct Visibility {
    pub visible: bool,
}

impl Default for Visibility {
    fn default() -> Self {
        Self { visible: true }
    }
}