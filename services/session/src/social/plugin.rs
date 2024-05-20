
use bevy_app::{App, Plugin, Update};
use bevy_ecs::schedule::IntoSystemConfigs;

use naia_bevy_server::ReceiveEvents;

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
            .add_systems(Update, SocialManager::update.in_set(ReceiveEvents));
    }
}