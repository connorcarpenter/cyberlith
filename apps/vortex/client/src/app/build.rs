use bevy_app::App;
use bevy_log::LogPlugin;

use render_api::Window;
use render_egui::EguiPlugin;

use crate::app::VortexPlugin;

pub fn build() -> App {
    let mut app = App::default();
    app
        // Bevy Plugins
        .add_plugin(LogPlugin::default())
        // Add Egui Plugin
        .add_plugin(EguiPlugin)
        // Add Game Plugin
        .add_plugin(VortexPlugin);
    app
}
