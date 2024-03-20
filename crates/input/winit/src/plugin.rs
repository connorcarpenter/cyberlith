use bevy_app::{App, Plugin, Update};

use crate::{WinitInput, InputEvent};

// Plugin
pub struct InputWinitPlugin;

impl Plugin for InputWinitPlugin {
    fn build(&self, app: &mut App) {
        app
            // Resources
            .insert_resource(WinitInput::new())
            // Events
            .add_event::<InputEvent>()
            // Systems
            .add_systems(Update, WinitInput::update);
    }
}
