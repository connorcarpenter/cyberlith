use bevy_app::{App, Plugin};

use crate::Input;

// Plugin
pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app
            // Resources
            .insert_resource(Input::new());
    }
}
