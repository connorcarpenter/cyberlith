use bevy_app::App;

use render_api::resources::WindowSettings;

use crate::window::Window;

pub fn runner_func(mut app: App) {
    // Get Window Settings
    let window_settings = app.world.remove_resource::<WindowSettings>().unwrap();

    // Run
    Window::run_render_loop(window_settings, app);
}
