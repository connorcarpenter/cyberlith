use bevy_app::{App, AppExit};

use render_api::resources::WindowSettings;

use crate::window::Window;

pub fn runner_func(mut app: App) -> AppExit {
    // Get Window Settings
    let window_settings = app.world_mut().remove_resource::<WindowSettings>().unwrap();

    // Run
    Window::run_render_loop(window_settings, app);

    AppExit::Success
}
