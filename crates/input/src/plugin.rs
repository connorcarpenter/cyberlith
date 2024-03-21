use bevy_app::{App, Plugin, Update};

use crate::{WinitInput, InputEvent};
use crate::gamepad::GilrsPlugin;

// Plugin
pub struct InputWinitPlugin;

impl Plugin for InputWinitPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(GilrsPlugin)
            // Resources
            .insert_resource(WinitInput::new())
            // Events
            .add_event::<InputEvent>()
            // Systems
            .add_systems(Update, WinitInput::update);
    }
}
