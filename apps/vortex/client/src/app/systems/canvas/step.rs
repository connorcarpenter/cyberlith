use bevy_ecs::system::{Query, ResMut};

use render_api::components::{Camera, Transform};

use crate::app::resources::{
    camera_manager::CameraManager, canvas::Canvas, shape_manager::ShapeManager,
};

pub fn update_camera(
    mut canvas: ResMut<Canvas>,
    mut camera_manager: ResMut<CameraManager>,
    mut shape_manager: ResMut<ShapeManager>,
    mut camera_q: Query<(&mut Camera, &mut Transform)>,
) {
    if canvas.update_visibility() {
        CameraManager::update_visibility(canvas.is_visible(), &mut camera_q);
    }
    if camera_manager.update_3d_camera(&mut camera_q) {
        shape_manager.recalculate_shapes();
    }
}
