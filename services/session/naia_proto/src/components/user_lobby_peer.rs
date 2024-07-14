use bevy_ecs::prelude::Component;

use naia_bevy_shared::Replicate;

#[derive(Component, Replicate)]
pub struct UserLobbyPeer;

impl UserLobbyPeer {
    pub fn new() -> Self {
        Self
    }
}
