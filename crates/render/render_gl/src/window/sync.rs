use bevy_app::AppExit;
use bevy_ecs::{
    event::EventReader,
    system::{NonSendMut, ResMut},
};

use render_api::{resources::Time, Window};

use crate::window::{FrameInput, OutgoingEvent};

pub fn sync(
    mut frame_input: NonSendMut<FrameInput<()>>,
    mut window: ResMut<Window>,
    mut time: ResMut<Time>,
    mut exit_event_reader: EventReader<AppExit>,
) {
    // update window res
    let mut update_window_resolution = false;
    if let Some(resolution) = window.get() {
        if resolution.logical_size.width != frame_input.logical_size.width
            || resolution.logical_size.height != frame_input.logical_size.height
        {
            update_window_resolution = true;
        }
    } else {
        update_window_resolution = true;
    }
    if update_window_resolution {
        window.set(frame_input.window_resolution());
    }

    // update elapsed time
    time.set_elapsed_ms(frame_input.elapsed_time_ms as f32);

    // read exit events
    for _ in exit_event_reader.read() {
        frame_input.outgoing_events.push(OutgoingEvent::Exit);
    }
}
