
use bevy_app::{App, Plugin};

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
        app.insert_resource(SocialManager::new());
    }
}