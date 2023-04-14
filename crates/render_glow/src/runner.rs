
use bevy_app::App;

use render_api::resources::WindowSettings;

use crate::window::{FrameInput, FrameOutput, Window};

pub fn three_d_runner(mut app: App) {
    // Get Window Settings
    let window_settings = app.world.remove_resource::<WindowSettings>().unwrap();

    // Create a Window
    let window = Window::new(window_settings).unwrap();

    // Start the main render loop
    window.render_loop(
        move |new_frame_input| // Begin a new frame with an updated frame input
            {
                // Insert FrameInput
                app
                    .world
                    .insert_non_send_resource(new_frame_input);

                // update app
                app.update();

                // Remove FrameInput
                let old_frame_input = app
                    .world
                    .remove_non_send_resource::<FrameInput<()>>().unwrap();

                // Returns default frame output to end the frame
                FrameOutput::from(old_frame_input)
            },
    );
}
