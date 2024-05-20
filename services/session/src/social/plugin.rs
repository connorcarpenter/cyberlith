
use bevy_app::{App, Plugin, Startup, Update};
use bevy_ecs::schedule::IntoSystemConfigs;

use naia_bevy_server::ReceiveEvents;

use crate::social::http_endpoints;
use super::social_manager::SocialManager;

pub struct SocialPlugin {

}

impl SocialPlugin {
    pub fn new(

    ) -> Self {
        Self {

        }
    }
}

impl Plugin for SocialPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(SocialManager::new())
            .add_systems(Startup, SocialManager::startup)
            .add_systems(
                Update, (
                    SocialManager::update,
                    http_endpoints::recv_patch_global_chat_messages_request,
                ).in_set(ReceiveEvents)
            );
    }
}