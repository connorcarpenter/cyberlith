use bevy_ecs::{system::{Query, ResMut}};

use render_api::components::{Camera, Transform};

use crate::app::resources::canvas_manager::CanvasManager;

pub fn step(
    mut canvas_state: ResMut<CanvasManager>,
    mut camera_q: Query<(&mut Camera, &mut Transform)>,
) {
    canvas_state.update_visibility(&mut camera_q);
    canvas_state.update_3d_camera(&mut camera_q);
}