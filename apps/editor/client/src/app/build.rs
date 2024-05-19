use bevy_app::App;

use input::InputPlugin;
use kernel::KernelPlugin;
use render_api::RenderApiPlugin;
use render_egui::EguiPlugin;
use render_gl::RenderGlPlugin;

use crate::app::VortexPlugin;

pub fn build() -> App {
    logging::initialize();

    let mut app = App::default();
    app
        // Add Kernel, with no cookiestore
        .add_plugins(KernelPlugin::new(None))
        // Add Render Plugins
        .add_plugins(RenderApiPlugin)
        .add_plugins(RenderGlPlugin)
        // Add Egui Plugin
        .add_plugins(EguiPlugin)
        // Add Input Plugin
        .add_plugins(InputPlugin)
        // Add Vortex Plugin
        .add_plugins(VortexPlugin);
    app
}
