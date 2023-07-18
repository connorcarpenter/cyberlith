use bevy_ecs::{entity::Entity, system::{Query, Res, ResMut}};

use input::Input;
use render_api::components::{Camera, Projection, Transform, Visibility};
use vortex_proto::components::Vertex3d;

use crate::app::resources::canvas_manager::CanvasManager;

pub fn sync(
    mut canvas_manager: ResMut<CanvasManager>,
    mut transform_q: Query<&mut Transform>,
    camera_q: Query<(&Camera, &Projection)>,
    vertex_3d_q: Query<(Entity, &Vertex3d)>,
    mut visibility_q: Query<&mut Visibility>,
) {
    canvas_manager.sync_vertices(&mut transform_q, &camera_q, &vertex_3d_q, &mut visibility_q);
}