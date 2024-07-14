use bevy_ecs::prelude::Component;

use naia_bevy_shared::Replicate;

#[derive(Component, Replicate)]
pub struct UserLobbyOwner;

impl UserLobbyOwner {
    pub fn new() -> Self {
        Self
    }
}
