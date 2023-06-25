use bevy_ecs::system::{NonSendMut, ResMut};

use input::Input;

use crate::window::FrameInput;

pub fn run(frame_input: NonSendMut<FrameInput<()>>, mut input: ResMut<Input>) {
    input.recv_events(&frame_input.incoming_events);
}
