use bevy_ecs::{entity::Entity, query::{With, Without}, system::{Query, ResMut}};

use render_api::components::{Camera, Projection, Transform, Visibility};
use vortex_proto::components::Vertex3d;

use crate::app::{components::{SelectCircle, Vertex2d}, resources::canvas_manager::CanvasManager};

pub fn sync(
    mut canvas_manager: ResMut<CanvasManager>,
    camera_q: Query<(&Camera, &Transform, &Projection), (Without<Vertex3d>, Without<Vertex2d>, Without<SelectCircle>)>,
    mut vertex_3d_q: Query<(Entity, &Vertex3d, &mut Transform), (Without<Vertex2d>, Without<SelectCircle>)>,
    mut vertex_2d_q: Query<&mut Transform, (With<Vertex2d>, Without<Vertex3d>, Without<SelectCircle>)>,
    mut select_circle_q: Query<(&mut Transform, &mut Visibility), (With<SelectCircle>, Without<Vertex3d>, Without<Vertex2d>)>,
) {
    canvas_manager.sync_vertices(&camera_q, &mut vertex_3d_q, &mut vertex_2d_q, &mut select_circle_q);
}