use std::sync::{Arc, RwLock};

use bevy_app::App;

use render_api::resources::WindowSettings;

use crate::window::{FrameInput, FrameOutput, Window};

pub static mut STOP_SIGNAL: Option<Arc<RwLock<StopSignal>>> = None;

pub struct StopSignal {
    pub stopped: bool,
}

use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        pub async fn wait_for_finish() {
            use gloo_timers::future::TimeoutFuture;

            let stop_signal = unsafe { STOP_SIGNAL.as_ref().unwrap().clone() };

            loop {

                if let Ok(stop_signal) = stop_signal.read() {
                    if stop_signal.stopped {
                        break;
                    }
                }
                TimeoutFuture::new(1_000).await;
            }
        }
    } else {
        pub async fn wait_for_finish() {
            panic!("should only call this in Wasm!");
        }
    }
}

pub fn runner_func(mut app: App) {
    // Get Window Settings
    let window_settings = app.world.remove_resource::<WindowSettings>().unwrap();

    // Create a Window
    let window = Window::new(window_settings).unwrap();

    let stop_signal = unsafe { STOP_SIGNAL.as_ref().unwrap().clone() };

    // Start the main render loop
    window.render_loop(
        stop_signal,
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
