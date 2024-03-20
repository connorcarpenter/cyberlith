use bevy_ecs::system::{NonSendMut, ResMut};

use input::winit::WinitInput;

use crate::window::FrameInput;

pub fn run(frame_input: NonSendMut<FrameInput<()>>, mut input: ResMut<WinitInput>) {
    input.recv_events(&frame_input.incoming_events);
}
