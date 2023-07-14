use bevy_ecs::{entity::Entity, query::{With, Without}, system::{Query, ResMut}};

use render_api::components::{Camera, Transform};
use vortex_proto::components::Vertex3d;

use crate::app::{components::Vertex2d, resources::canvas_state::CanvasState};

pub fn sync(
    mut canvas_state: ResMut<CanvasState>,
    mut vertex_3d_query: Query<(Entity, &Vertex3d, &mut Transform), Without<Vertex2d>>,
    mut vertex_2d_query: Query<&mut Transform, With<Vertex2d>>,
) {
    for (entity, vertex_3d, mut transform) in vertex_3d_query.iter_mut() {
        transform.translation.x = vertex_3d.x().into();
        transform.translation.y = vertex_3d.y().into();
        transform.translation.z = vertex_3d.z().into();

        if let Some(vertex_2d_entity) = canvas_state.vertex_entity_3d_to_2d(&entity) {
            if let Ok(mut vertex_2d_transform) = vertex_2d_query.get_mut(*vertex_2d_entity) {
                vertex_2d_transform.translation.x = vertex_3d.x().into();
                vertex_2d_transform.translation.y = vertex_3d.y().into();
                vertex_2d_transform.translation.z = vertex_3d.z().into();
            }
        }
    }
}