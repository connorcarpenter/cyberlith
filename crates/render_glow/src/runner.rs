use std::default::Default;

use bevy_app::App;

use crate::window::{FrameInput, FrameOutput, Window, WindowSettings};

pub fn three_d_runner(mut app: App) {
    // Create a Window
    // TODO: bring these settings into the app
    let window = Window::new(WindowSettings {
        title: "Triangle!".to_string(),
        max_size: Some((1280, 720)),
        ..Default::default()
    })
    .unwrap();

    // Start the main render loop
    window.render_loop(
        move |frame_input| // Begin a new frame with an updated frame input
            {
                // Insert FrameInput
                app
                    .world
                    .insert_non_send_resource(frame_input);

                // update app
                app.update();

                // Remove FrameInput
                app
                    .world
                    .remove_non_send_resource::<FrameInput<()>>();

                // Returns default frame output to end the frame
                FrameOutput::default()
            },
    );
}
