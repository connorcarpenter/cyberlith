use bevy_ecs::{
    entity::Entity,
    system::{Commands, Query, Res, ResMut},
};

use input::Input;

use render_api::{
    base::{CpuMaterial, CpuMesh},
    components::{Camera, Projection, Transform, Visibility},
    Assets,
};

use vortex_proto::components::{EdgeAngle, Vertex3d};

use crate::app::{
    components::{Compass, Edge2dLocal, Edge3dLocal, FaceIcon2d, OwnedByFileLocal},
    resources::{
        camera_manager::CameraManager, canvas::Canvas, shape_manager::ShapeManager,
        tab_manager::TabManager,
    },
};

pub fn sync_vertices(
    tab_manager: ResMut<TabManager>,
    canvas: Res<Canvas>,
    camera_manager: Res<CameraManager>,
    mut shape_manager: ResMut<ShapeManager>,

    compass_q: Query<&Compass>,
    camera_q: Query<(&Camera, &Projection)>,

    mut transform_q: Query<&mut Transform>,
    owned_by_tab_q: Query<&OwnedByFileLocal>,

    mut vertex_3d_q: Query<(Entity, &mut Vertex3d)>,
    edge_2d_q: Query<(Entity, &Edge2dLocal)>,
    edge_3d_q: Query<(Entity, &Edge3dLocal, Option<&EdgeAngle>)>,
    face_2d_q: Query<(Entity, &FaceIcon2d)>,
) {
    if !canvas.is_visible() {
        return;
    }
    let Some(current_tab_state) = tab_manager.current_tab_state() else {
        return;
    };

    if let Some(current_tab_entity) = tab_manager.current_tab_entity() {
        let current_tab_camera_state = &current_tab_state.camera_state;

        shape_manager.sync_shapes(
            &camera_manager,
            current_tab_camera_state,
            *current_tab_entity,
            &camera_q,
            &compass_q,
            &mut transform_q,
            &owned_by_tab_q,
            &mut vertex_3d_q,
            &edge_2d_q,
            &edge_3d_q,
            &face_2d_q,
        );
    }
}

pub fn process_faces(
    mut commands: Commands,
    camera_manager: Res<CameraManager>,
    mut shape_manager: ResMut<ShapeManager>,
    mut meshes: ResMut<Assets<CpuMesh>>,
    mut materials: ResMut<Assets<CpuMaterial>>,
) {
    shape_manager.process_new_faces(&mut commands, &camera_manager, &mut meshes, &mut materials);
}

pub fn update_select_line(
    canvas: Res<Canvas>,
    tab_manager: ResMut<TabManager>,
    input: Res<Input>,
    mut shape_manager: ResMut<ShapeManager>,
    mut transform_q: Query<&mut Transform>,
    mut visibility_q: Query<&mut Visibility>,
) {
    if !canvas.is_visible() {
        return;
    }
    let Some(current_tab_state) = tab_manager.current_tab_state() else {
        return;
    };

    let current_tab_camera_state = &current_tab_state.camera_state;

    shape_manager.update_select_line(
        input.mouse_position(),
        &canvas,
        current_tab_camera_state,
        &mut transform_q,
        &mut visibility_q,
    );
}
