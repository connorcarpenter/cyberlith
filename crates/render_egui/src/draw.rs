use bevy_ecs::system::NonSendMut;

use render_glow::window::FrameInput;

pub fn draw(frame_input: NonSendMut<FrameInput<()>>) {}
