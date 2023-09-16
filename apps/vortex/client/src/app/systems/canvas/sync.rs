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
        face_manager::FaceManager, file_manager::FileManager, input_manager::InputManager,
        tab_manager::TabManager, vertex_manager::VertexManager,
    },
};

pub fn sync_vertices(
    file_manager: Res<FileManager>,
    tab_manager: ResMut<TabManager>,
    mut canvas: ResMut<Canvas>,
    camera_manager: Res<CameraManager>,
    compass: Res<Compass>,
    vertex_manager: Res<VertexManager>,
    edge_manager: Res<EdgeManager>,
    mut input_manager: ResMut<InputManager>,

    local_shape_q: Query<&LocalShape>,
    camera_q: Query<(&Camera, &Projection)>,

    mut visibility_q: Query<&mut Visibility>,
    mut transform_q: Query<&mut Transform>,
    owned_by_q: Query<&OwnedByFileLocal>,

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

    if let Some(current_tab_file_entity) = tab_manager.current_tab_entity() {
        let camera_state = &current_tab_state.camera_state;

        let should_sync_shapes = canvas.should_sync_shapes(&camera_manager);
        if should_sync_shapes {
            input_manager.queue_resync_hover_ui();
            input_manager.queue_resync_selection_ui();
            let camera_3d = camera_manager.camera_3d_entity().unwrap();
            let camera_3d_scale = camera_state.camera_3d_scale();

            compass.sync_compass(&camera_3d, camera_state, &mut vertex_3d_q, &transform_q);
            vertex_manager.sync_vertices(
                &file_manager,
                &camera_3d,
                camera_3d_scale,
                &camera_q,
                &vertex_3d_q,
                &mut transform_q,
                &owned_by_q,
                &local_shape_q,
                *current_tab_file_entity,
            );
            EdgeManager::sync_2d_edges(
                &file_manager,
                &vertex_manager,
                &edge_2d_q,
                &mut transform_q,
                &owned_by_q,
                &local_shape_q,
                *current_tab_file_entity,
                camera_3d_scale,
            );
            edge_manager.sync_3d_edges(
                &file_manager,
                &edge_3d_q,
                &mut transform_q,
                &owned_by_q,
                &mut visibility_q,
                &local_shape_q,
                *current_tab_file_entity,
                camera_3d_scale,
            );
            FaceManager::sync_2d_faces(
                &file_manager,
                &face_2d_q,
                &mut transform_q,
                &owned_by_q,
                *current_tab_file_entity,
                camera_3d_scale,
            );
            input_manager.sync_hover_shape_scale(&mut transform_q, camera_3d_scale);
        }
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
    file_manager: Res<FileManager>,
    tab_manager: Res<TabManager>,
    input: Res<Input>,
    mut input_manager: ResMut<InputManager>,
    mut transform_q: Query<&mut Transform>,
    mut visibility_q: Query<&mut Visibility>,
) {
    if !canvas.is_visible() {
        return;
    }

    input_manager.sync_selection_ui(
        &canvas,
        &file_manager,
        &tab_manager,
        &mut transform_q,
        &mut visibility_q,
        input.mouse_position(),
    );
}
