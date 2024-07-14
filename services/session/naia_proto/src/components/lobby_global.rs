use bevy_ecs::{prelude::Component};

use naia_bevy_shared::Replicate;

#[derive(Component, Replicate)]
pub struct LobbyGlobal;

impl LobbyGlobal {
    pub fn new() -> Self {
        Self
    }
}
