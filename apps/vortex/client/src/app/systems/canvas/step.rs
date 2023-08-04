use bevy_ecs::system::{Query, ResMut};

use render_api::components::{Camera, Transform};

use crate::app::resources::{camera_manager::CameraManager, canvas::Canvas, canvas_manager::CanvasManager};

pub fn step(
    mut canvas: ResMut<Canvas>,
    mut camera_manager: ResMut<CameraManager>,
    mut canvas_manager: ResMut<CanvasManager>,
    mut camera_q: Query<(&mut Camera, &mut Transform)>,
) {
    if canvas.update_visibility() {
        CameraManager::update_visibility(canvas.is_visible(), &mut camera_q);
    }
    if camera_manager.update_3d_camera(&mut camera_q) {
        canvas_manager.recalculate_vertices();
    }
}
