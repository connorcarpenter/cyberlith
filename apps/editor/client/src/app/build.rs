use bevy_app::App;
use bevy_log::LogPlugin;

use input::InputPlugin;
use render_api::Window;
use render_egui::EguiPlugin;

use crate::app::EditorPlugin;

pub fn build() -> App {
    let mut app = App::default();
    app
        // Bevy Plugins
        .add_plugin(LogPlugin::default())
        // Add Input Plugin
        .add_plugin(InputPlugin)
        // Add Egui Plugin
        .add_plugin(EguiPlugin)
        // Add Game Plugin
        .add_plugin(EditorPlugin);
    app
}
