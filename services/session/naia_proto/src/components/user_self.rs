use bevy_ecs::prelude::Component;

use naia_bevy_shared::Replicate;

#[derive(Component, Replicate)]
pub struct UserSelf;

impl UserSelf {
    pub fn new() -> Self {
        Self
    }
}
