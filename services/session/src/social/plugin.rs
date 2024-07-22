use bevy_app::{App, Plugin, Startup, Update};
use bevy_ecs::schedule::IntoSystemConfigs;

use naia_bevy_server::ReceiveEvents;

use super::social_manager::SocialManager;
use crate::social::http_endpoints;

pub struct SocialPlugin;

impl Plugin for SocialPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SocialManager::new())
            .add_systems(Startup, SocialManager::startup)
            .add_systems(
                Update,
                (
                    SocialManager::update,
                    http_endpoints::recv_patch_users_request,
                    http_endpoints::recv_patch_global_chat_messages_request,
                    http_endpoints::recv_patch_match_lobby_request,
                    http_endpoints::recv_world_connect,
                )
                    .in_set(ReceiveEvents),
            );
    }
}
