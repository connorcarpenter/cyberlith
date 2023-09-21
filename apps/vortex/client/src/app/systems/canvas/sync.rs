use bevy_ecs::{
    entity::Entity,
    query::With,
    system::{Commands, Query, Res, ResMut},
};

use input::Input;

use render_api::{
    base::{CpuMaterial, CpuMesh},
    components::{Camera, Projection, Transform, Visibility},
    Assets,
};

use vortex_proto::components::{
    AnimRotation, EdgeAngle, FileExtension, ShapeName, Vertex3d, VertexRoot,
};

use crate::app::{
    components::{Edge2dLocal, Edge3dLocal, FaceIcon2d, LocalAnimRotation, LocalShape},
    resources::{
        animation_manager::AnimationManager, camera_manager::CameraManager, canvas::Canvas,
        compass::Compass, edge_manager::EdgeManager, face_manager::FaceManager,
        file_manager::FileManager, input_manager::InputManager, tab_manager::TabManager,
        vertex_manager::VertexManager,
    },
};

pub fn queue_resyncs(
    mut canvas: ResMut<Canvas>,
    tab_manager: Res<TabManager>,
    camera_manager: Res<CameraManager>,
    mut compass: ResMut<Compass>,
    mut vertex_manager: ResMut<VertexManager>,
    mut edge_manager: ResMut<EdgeManager>,
    mut input_manager: ResMut<InputManager>,
    mut face_manager: ResMut<FaceManager>,
) {
    if !canvas.is_visible() {
        return;
    }
    if tab_manager.current_tab_entity().is_none() {
        return;
    }
    if tab_manager.current_tab_state().is_none() {
        return;
    };
    if camera_manager.camera_3d_entity().is_none() {
        return;
    }

    let should_sync_shapes = canvas.should_sync_shapes();
    if should_sync_shapes {
        input_manager.queue_resync_hover_ui();
        input_manager.queue_resync_selection_ui();
        compass.queue_resync();
        vertex_manager.queue_resync();
        edge_manager.queue_resync();
        face_manager.queue_resync();
    }
}

pub fn sync_compass(
    canvas: Res<Canvas>,
    tab_manager: Res<TabManager>,
    camera_manager: Res<CameraManager>,
    mut compass: ResMut<Compass>,
    transform_q: Query<&Transform>,
    mut vertex_3d_q: Query<(Entity, &mut Vertex3d)>,
) {
    let Some(current_tab_state) = tab_manager.current_tab_state() else {
        return;
    };
    let camera_state = &current_tab_state.camera_state;
    let camera_3d = camera_manager.camera_3d_entity().unwrap();

    compass.sync_compass(&canvas, &camera_3d, camera_state, &mut vertex_3d_q, &transform_q);
}

pub fn sync_vertices(
    file_manager: Res<FileManager>,
    tab_manager: Res<TabManager>,
    canvas: Res<Canvas>,
    camera_manager: Res<CameraManager>,
    compass: Res<Compass>,
    mut vertex_manager: ResMut<VertexManager>,
    animation_manager: Res<AnimationManager>,
    local_shape_q: Query<&LocalShape>,
    camera_q: Query<(&Camera, &Projection)>,
    mut transform_q: Query<&mut Transform>,
    visibility_q: Query<&Visibility>,
    vertex_3d_q: Query<(Entity, &Vertex3d)>,
    name_q: Query<&ShapeName>,
    mut rotation_q: Query<(&AnimRotation, &mut LocalAnimRotation)>,
    root_q: Query<Entity, With<VertexRoot>>,
) {
    if !canvas.is_visible() {
        return;
    }
    let Some(current_tab_state) = tab_manager.current_tab_state() else {
        return;
    };
    let current_file_entity = tab_manager.current_tab_entity().unwrap();
    let file_extension = file_manager.get_file_type(current_file_entity);

    let camera_3d = camera_manager.camera_3d_entity().unwrap();
    let camera_state = &current_tab_state.camera_state;
    let camera_3d_scale = camera_state.camera_3d_scale();

    let did_sync = match file_extension {
        FileExtension::Skel | FileExtension::Mesh => {
            vertex_manager.sync_vertices_3d(&vertex_3d_q, &mut transform_q, &visibility_q)
        }
        FileExtension::Anim => vertex_manager.sync_vertices_3d_anim(
            &animation_manager,
            &compass,
            &vertex_3d_q,
            &mut transform_q,
            &visibility_q,
            &name_q,
            &mut rotation_q,
            &root_q,
        ),
        _ => false,
    };
    if did_sync {
        vertex_manager.sync_vertices_2d(
            &camera_3d,
            camera_3d_scale,
            &camera_q,
            &vertex_3d_q,
            &mut transform_q,
            &visibility_q,
            &local_shape_q,
        );
    }
}

pub fn sync_edges(
    tab_manager: Res<TabManager>,
    canvas: Res<Canvas>,
    mut edge_manager: ResMut<EdgeManager>,
    local_shape_q: Query<&LocalShape>,
    mut visibility_q: Query<&mut Visibility>,
    mut transform_q: Query<&mut Transform>,
    edge_2d_q: Query<(Entity, &Edge2dLocal)>,
    edge_3d_q: Query<(Entity, &Edge3dLocal, Option<&EdgeAngle>)>,
) {
    if !canvas.is_visible() {
        return;
    }
    let Some(current_tab_state) = tab_manager.current_tab_state() else {
        return;
    };
    let camera_state = &current_tab_state.camera_state;
    let camera_3d_scale = camera_state.camera_3d_scale();

    edge_manager.sync_edges(
        &edge_2d_q,
        &edge_3d_q,
        &mut transform_q,
        &mut visibility_q,
        &local_shape_q,
        camera_3d_scale,
    );
}

pub fn sync_faces(
    tab_manager: Res<TabManager>,
    canvas: Res<Canvas>,
    mut face_manager: ResMut<FaceManager>,
    mut transform_q: Query<&mut Transform>,
    visibility_q: Query<&Visibility>,
    face_2d_q: Query<(Entity, &FaceIcon2d)>,
) {
    if !canvas.is_visible() {
        return;
    }
    let Some(current_tab_state) = tab_manager.current_tab_state() else {
        return;
    };
    let camera_state = &current_tab_state.camera_state;
    let camera_3d_scale = camera_state.camera_3d_scale();

    face_manager.sync_2d_faces(&face_2d_q, &mut transform_q, &visibility_q, camera_3d_scale);
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
