use bevy_app::App;
use bevy_log::LogPlugin;

use input::InputPlugin;
use render_api::RenderApiPlugin;
use render_egui::EguiPlugin;
use render_glow::RenderGlowPlugin;

use crate::app::VortexPlugin;

pub fn build() -> App {
    let mut app = App::default();
    app
        // Bevy Plugins
        .add_plugin(LogPlugin::default())
        // Add Render Plugins
        .add_plugin(RenderApiPlugin)
        .add_plugin(RenderGlowPlugin)
        // Add Egui Plugin
        .add_plugin(EguiPlugin)
        // Add Input Plugin
        .add_plugin(InputPlugin)
        // Add Vortex Plugin
        .add_plugin(VortexPlugin);
    app
}
