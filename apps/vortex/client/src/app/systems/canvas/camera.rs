use bevy_ecs::{change_detection::DetectChanges, system::{Query, Res}};

use render_api::components::Camera;

use crate::app::resources::canvas_state::CanvasState;

pub fn update_cameras(
    canvas_state: Res<CanvasState>,
    mut camera_q: Query<&mut Camera>,
) {
    if !canvas_state.is_changed() {
        return;
    }

    canvas_state.update_all_camera_visibility(&mut camera_q);
}