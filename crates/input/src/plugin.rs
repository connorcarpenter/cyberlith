use bevy_app::{App, Plugin};

use input_winit::InputWinitPlugin;

// Plugin
pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputWinitPlugin);
    }
}
