use bevy_ecs::system::{Query, ResMut};

use render_api::components::{Camera, Projection, Transform};

use crate::app::resources::{
    camera_manager::CameraManager, canvas::Canvas, shape_manager::ShapeManager,
    tab_manager::TabManager,
};

pub fn update_camera(
    mut canvas: ResMut<Canvas>,
    mut camera_manager: ResMut<CameraManager>,
    tab_manager: ResMut<TabManager>,
    mut camera_q: Query<(&mut Camera, &mut Projection, &mut Transform)>,
) {
    if canvas.update_visibility() {
        CameraManager::update_visibility(canvas.is_visible(), &mut camera_q);
    }
    let Some(current_tab_state) = tab_manager.current_tab_state() else {
        return;
    };
    let current_tab_camera_state = &current_tab_state.camera_state;
    if camera_manager.update_3d_camera(current_tab_camera_state, &mut camera_q) {
        canvas.queue_resync_shapes();
    }
}
