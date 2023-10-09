use bevy_ecs::system::{Query, Res, ResMut};

use render_api::components::{Camera, Projection, Transform};

use vortex_proto::components::FileExtension;

use crate::app::resources::{
    animation_manager::AnimationManager, camera_manager::CameraManager, canvas::Canvas,
    file_manager::FileManager, tab_manager::TabManager, edge_manager::EdgeManager, input_manager::InputManager, vertex_manager::VertexManager
};

pub fn update_camera(
    mut canvas: ResMut<Canvas>,
    file_manager: Res<FileManager>,
    mut camera_manager: ResMut<CameraManager>,
    tab_manager: ResMut<TabManager>,
    animation_manager: Res<AnimationManager>,
    mut camera_q: Query<(&mut Camera, &mut Projection, &mut Transform)>,
) {
    if canvas.update_visibility() {
        CameraManager::update_visibility(canvas.is_visible(), &mut camera_q);
    }
    let Some(current_tab_entity) = tab_manager.current_tab_entity() else {
        return;
    };
    let file_ext = file_manager.get_file_type(current_tab_entity);
    if file_ext == FileExtension::Anim {
        if animation_manager.is_framing() {
            return;
        }
    }

    let Some(current_tab_state) = tab_manager.current_tab_state() else {
        return;
    };
    let current_tab_camera_state = &current_tab_state.camera_state;
    if camera_manager.update_3d_camera(current_tab_camera_state, &mut camera_q) {
        canvas.queue_resync_shapes();
    }
}

pub fn update_tab_content_focus(
    mut canvas: ResMut<Canvas>,
    tab_manager: Res<TabManager>,
    mut input_manager: ResMut<InputManager>,
    mut vertex_manager: ResMut<VertexManager>,
    mut edge_manager: ResMut<EdgeManager>,
    mut animation_manager: ResMut<AnimationManager>,
) {
    canvas.update_sync_focus(&tab_manager, &mut input_manager, &mut vertex_manager, &mut edge_manager, &mut animation_manager);
}
