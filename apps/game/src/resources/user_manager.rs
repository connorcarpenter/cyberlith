
use bevy_ecs::system::{Resource};

#[derive(Resource)]
pub struct UserManager {}

impl Default for UserManager {
    fn default() -> Self {
        Self {}
    }
}

impl UserManager {}
