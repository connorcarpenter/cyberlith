use bevy_ecs::system::{NonSendMut, ResMut};

use render_api::{Window, resources::Time};

use crate::window::FrameInput;

pub fn sync(frame_input: NonSendMut<FrameInput<()>>, mut window: ResMut<Window>, mut time: ResMut<Time>) {
    let mut update = false;
    if let Some(resolution) = window.get() {
        if resolution.logical_size.width != frame_input.logical_size.width
            || resolution.logical_size.height != frame_input.logical_size.height
        {
            update = true;
        }
    } else {
        update = true;
    }
    if update {
        window.set(frame_input.window_resolution());
    }

    time.set_elapsed(frame_input.elapsed_time as f32);
}
