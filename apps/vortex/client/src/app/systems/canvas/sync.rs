use bevy_ecs::{
    entity::Entity,
    system::{Commands, Query, Res, ResMut},
};

use input::Input;

use render_api::{Assets, base::{CpuMaterial, CpuMesh}, components::{Camera, Projection, Transform, Visibility}};

use vortex_proto::components::{OwnedByFile, Vertex3d};

use crate::app::{
    components::{FaceIcon2d, Compass, Edge2dLocal, Edge3dLocal},
    resources::{
        camera_manager::CameraManager, canvas::Canvas, shape_manager::ShapeManager,
        tab_manager::TabManager,
    },
};
use crate::app::components::OwnedByFileLocal;

pub fn sync_vertices(
    tab_manager: Res<TabManager>,
    canvas: Res<Canvas>,
    camera_manager: Res<CameraManager>,
    mut shape_manager: ResMut<ShapeManager>,
    mut transform_q: Query<&mut Transform>,
    compass_q: Query<&Compass>,
    camera_q: Query<(&Camera, &Projection)>,
    mut vertex_3d_q: Query<(Entity, &mut Vertex3d)>,
    edge_2d_q: Query<(Entity, &Edge2dLocal)>,
    edge_3d_q: Query<(Entity, &Edge3dLocal)>,
    face_2d_q: Query<(Entity, &FaceIcon2d)>,
    owned_by_tab_q: Query<&OwnedByFileLocal>,
) {
    if !canvas.is_visible() {
        return;
    }
    shape_manager.sync_vertices(
        &camera_manager,
        tab_manager.current_tab_entity(),
        &compass_q,
        &mut transform_q,
        &camera_q,
        &mut vertex_3d_q,
        &edge_2d_q,
        &edge_3d_q,
        &face_2d_q,
        &owned_by_tab_q,
    );
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
    camera_manager: Res<CameraManager>,
    input: Res<Input>,
    mut shape_manager: ResMut<ShapeManager>,
    mut transform_q: Query<&mut Transform>,
    mut visibility_q: Query<&mut Visibility>,
) {
    if !canvas.is_visible() {
        return;
    }

    shape_manager.update_select_line(
        input.mouse_position(),
        &camera_manager,
        &mut transform_q,
        &mut visibility_q,
    );
}
