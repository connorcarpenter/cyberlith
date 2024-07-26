use bevy_app::{App, Plugin};

use crate::social::LobbyManager;

pub struct SocialPlugin;

impl Plugin for SocialPlugin {
    fn build(&self, app: &mut App) {
        app
            // Resource
            .init_resource::<LobbyManager>();
    }
}
