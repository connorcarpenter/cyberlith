use bevy_ecs::{
    entity::Entity,
    system::{Query, ResMut, Res},
};

use render_api::components::{Camera, Projection, Transform};
use vortex_proto::components::{OwnedByTab, Vertex3d};

use crate::app::{
    components::{Compass, Edge2d, Edge3d},
    resources::{canvas::Canvas, canvas_manager::CanvasManager, tab_manager::TabManager},
};

pub fn sync(
    tab_manager: Res<TabManager>,
    canvas: Res<Canvas>,
    mut canvas_manager: ResMut<CanvasManager>,
    mut transform_q: Query<(&mut Transform, Option<&Compass>)>,
    camera_q: Query<(&Camera, &Projection)>,
    vertex_3d_q: Query<(Entity, &Vertex3d)>,
    edge_2d_q: Query<(Entity, &Edge2d)>,
    edge_3d_q: Query<(Entity, &Edge3d)>,
    owned_by_tab_q: Query<&OwnedByTab>,
) {
    if !canvas.is_visible() {
        return;
    }
    canvas_manager.sync_vertices(
        tab_manager.current_tab_id(),
        &mut transform_q,
        &camera_q,
        &vertex_3d_q,
        &edge_2d_q,
        &edge_3d_q,
        &owned_by_tab_q,
    );
}
