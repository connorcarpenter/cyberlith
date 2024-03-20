use bevy_app::{App, Plugin, Update};

use crate::{Input, InputEvent};

// Plugin
pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app
            // Resources
            .insert_resource(Input::new())
            // Events
            .add_event::<InputEvent>()
            // Systems
            .add_systems(Update, Input::update);
    }
}
