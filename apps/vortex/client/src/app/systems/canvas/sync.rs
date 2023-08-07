use bevy_ecs::{
    entity::Entity,
    system::{Query, Res, ResMut},
};

use input::Input;
use render_api::components::{Camera, Projection, Transform, Visibility};
use vortex_proto::components::{OwnedByTab, Vertex3d};

use crate::app::{
    components::{Compass, Edge2dLocal, Edge3dLocal},
    resources::{
        camera_manager::CameraManager, canvas::Canvas, tab_manager::TabManager,
        vertex_manager::VertexManager,
    },
};

pub fn sync_vertices(
    tab_manager: Res<TabManager>,
    canvas: Res<Canvas>,
    camera_manager: Res<CameraManager>,
    mut vertex_manager: ResMut<VertexManager>,
    mut transform_q: Query<&mut Transform>,
    compass_q: Query<&Compass>,
    camera_q: Query<(&Camera, &Projection)>,
    mut vertex_3d_q: Query<(Entity, &mut Vertex3d)>,
    edge_2d_q: Query<(Entity, &Edge2dLocal)>,
    edge_3d_q: Query<(Entity, &Edge3dLocal)>,
    owned_by_tab_q: Query<&OwnedByTab>,
) {
    if !canvas.is_visible() {
        return;
    }
    vertex_manager.sync_vertices(
        &camera_manager,
        tab_manager.current_tab_id(),
        &compass_q,
        &mut transform_q,
        &camera_q,
        &mut vertex_3d_q,
        &edge_2d_q,
        &edge_3d_q,
        &owned_by_tab_q,
    );
}

pub fn update_select_line(
    canvas: Res<Canvas>,
    camera_manager: Res<CameraManager>,
    input: Res<Input>,
    mut vertex_manager: ResMut<VertexManager>,
    mut transform_q: Query<&mut Transform>,
    mut visibility_q: Query<&mut Visibility>,
) {
    if !canvas.is_visible() {
        return;
    }

    vertex_manager.update_select_line(
        input.mouse_position(),
        &camera_manager,
        &mut transform_q,
        &mut visibility_q
    );
}
