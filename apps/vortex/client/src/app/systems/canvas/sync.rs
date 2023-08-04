use bevy_ecs::{
    entity::Entity,
    system::{Query, Res, ResMut},
};

use render_api::components::{Camera, Projection, Transform};
use vortex_proto::components::{OwnedByTab, Vertex3d};

use crate::app::{
    components::{Compass, Edge2d, Edge3d},
    resources::{
        camera_manager::CameraManager, canvas::Canvas, tab_manager::TabManager,
        vertex_manager::VertexManager,
    },
};

pub fn sync(
    tab_manager: Res<TabManager>,
    canvas: Res<Canvas>,
    mut camera_manager: ResMut<CameraManager>,
    mut vertex_manager: ResMut<VertexManager>,
    mut transform_q: Query<(&mut Transform, Option<&Compass>)>,
    camera_q: Query<(&Camera, &Projection)>,
    mut vertex_3d_q: Query<(Entity, &mut Vertex3d)>,
    edge_2d_q: Query<(Entity, &Edge2d)>,
    edge_3d_q: Query<(Entity, &Edge3d)>,
    owned_by_tab_q: Query<&OwnedByTab>,
) {
    if !canvas.is_visible() {
        return;
    }
    vertex_manager.sync_vertices(
        &mut camera_manager,
        tab_manager.current_tab_id(),
        &mut transform_q,
        &camera_q,
        &mut vertex_3d_q,
        &edge_2d_q,
        &edge_3d_q,
        &owned_by_tab_q,
    );
}
