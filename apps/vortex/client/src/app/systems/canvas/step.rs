use bevy_ecs::{system::{Query, ResMut}};

use render_api::components::Camera;

use crate::app::resources::canvas_state::CanvasState;

pub fn step(
    mut canvas_state: ResMut<CanvasState>,
    mut camera_q: Query<&mut Camera>,
) {
    canvas_state.update(&mut camera_q);
}