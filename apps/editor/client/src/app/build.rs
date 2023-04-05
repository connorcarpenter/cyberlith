use bevy_app::App;
use bevy_log::LogPlugin;

use render_api::Window;
use render_egui::EguiPlugin;

use crate::app::EditorPlugin;

pub fn build() -> App {
    let mut app = App::default();
    app
        // Bevy Plugins
        .add_plugin(LogPlugin::default())
        // Insert Window Resource
        // TODO: find out how to get window height & width
        .insert_resource(Window::new(1280, 720))
        // Add Egui Plugin
        .add_plugin(EguiPlugin)
        // Add Game Plugin
        .add_plugin(EditorPlugin);
    app
}
