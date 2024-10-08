use bevy_app::{App, Plugin, Update};

use crate::{gamepad::GamepadPlugin, Input, InputEvent};

// Plugin
pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app
            // Plugins
            .add_plugins(GamepadPlugin)
            // Resources
            .insert_resource(Input::new())
            // Events
            .add_event::<InputEvent>()
            // Systems
            .add_systems(Update, Input::process_clipboard_task_keys);
    }
}
