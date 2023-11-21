use bevy_app::App;
use bevy_log::LogPlugin;

use input::InputPlugin;
use render_api::RenderApiPlugin;
use render_egui::EguiPlugin;
use render_gl::RenderglPlugin;

use crate::app::VortexPlugin;

pub fn build() -> App {
    let mut app = App::default();
    app
        // Bevy Plugins
        .add_plugins(LogPlugin::default())
        // Add Render Plugins
        .add_plugins(RenderApiPlugin)
        .add_plugins(RenderglPlugin)
        // Add Egui Plugin
        .add_plugins(EguiPlugin)
        // Add Input Plugin
        .add_plugins(InputPlugin)
        // Add Vortex Plugin
        .add_plugins(VortexPlugin);
    app
}
