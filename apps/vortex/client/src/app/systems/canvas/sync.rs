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
    components::{Edge2dLocal, Edge3dLocal, FaceIcon2d, LocalShape, OwnedByFileLocal},
    resources::{
        camera_manager::CameraManager, canvas::Canvas, compass::Compass, edge_manager::EdgeManager,
        face_manager::FaceManager, input_manager::InputManager, tab_manager::TabManager,
        vertex_manager::VertexManager,
    },
};

pub fn sync_vertices(
    tab_manager: ResMut<TabManager>,
    mut canvas: ResMut<Canvas>,
    camera_manager: Res<CameraManager>,
    compass: Res<Compass>,
    vertex_manager: Res<VertexManager>,
    edge_manager: Res<EdgeManager>,
    mut input_manager: ResMut<InputManager>,

    compass_q: Query<&LocalShape>,
    camera_q: Query<(&Camera, &Projection)>,

    mut visibility_q: Query<&mut Visibility>,
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

        canvas.sync_shapes(
            &camera_manager,
            current_tab_camera_state,
            &compass,
            *current_tab_entity,
            &vertex_manager,
            &edge_manager,
            &mut input_manager,
            &camera_q,
            &compass_q,
            &mut visibility_q,
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
    mut canvas: ResMut<Canvas>,
    camera_manager: Res<CameraManager>,
    mut vertex_manager: ResMut<VertexManager>,
    mut edge_manager: ResMut<EdgeManager>,
    mut face_manager: ResMut<FaceManager>,
    mut meshes: ResMut<Assets<CpuMesh>>,
    mut materials: ResMut<Assets<CpuMaterial>>,
) {
    face_manager.process_new_faces(
        &mut commands,
        &mut canvas,
        &camera_manager,
        &mut vertex_manager,
        &mut edge_manager,
        &mut meshes,
        &mut materials,
    );
}

pub fn update_select_line(
    canvas: Res<Canvas>,
    tab_manager: ResMut<TabManager>,
    input: Res<Input>,
    mut input_manager: ResMut<InputManager>,
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

    input_manager.sync_selection_ui(
        input.mouse_position(),
        &canvas,
        current_tab_camera_state,
        &mut transform_q,
        &mut visibility_q,
    );
}
