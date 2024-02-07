use bevy_ecs::{
    prelude::{Query, World},
    system::{Res, ResMut, SystemState},
};
use bevy_log::info;

use naia_bevy_client::Client;

use render_api::{base::CpuMesh, components::Transform};

use editor_proto::components::{Face3d, Vertex3d};
use storage::{Assets, Handle};

use crate::app::{
    plugin::Main,
    resources::{
        action::shape::ShapeAction, canvas::Canvas, face_manager::FaceManager,
        vertex_manager::VertexManager,
    },
};

pub(crate) fn execute(world: &mut World, action: ShapeAction) -> Vec<ShapeAction> {
    let ShapeAction::MoveVertex(vertex_2d_entity, old_position, new_position, already_moved) = action else {
        panic!("Expected MoveVertex");
    };

    info!(
        "MoveVertex({:?}, _, _, {})",
        vertex_2d_entity, already_moved
    );
    let mut system_state: SystemState<(
        Client<Main>,
        ResMut<Canvas>,
        ResMut<Assets<CpuMesh>>,
        ResMut<VertexManager>,
        Res<FaceManager>,
        Query<&Handle<CpuMesh>>,
        Query<&Face3d>,
        Query<&mut Transform>,
        Query<&mut Vertex3d>,
    )> = SystemState::new(world);
    let (
        client,
        mut canvas,
        mut meshes,
        vertex_manager,
        face_manager,
        mesh_handle_q,
        face_3d_q,
        mut transform_q,
        mut vertex_3d_q,
    ) = system_state.get_mut(world);

    let vertex_3d_entity = vertex_manager
        .vertex_entity_2d_to_3d(&vertex_2d_entity)
        .unwrap();

    if !already_moved {
        // MoveVertex action happens after the vertex has already been moved, so we wouldn't need to do anything here ..
        // BUT we do need to update the vertex_3d's position here in order to apply when undo/redo is executed
        let Ok(mut vertex_3d) = vertex_3d_q.get_mut(vertex_3d_entity) else {
            panic!("Failed to get Vertex3d for vertex entity {:?}!", vertex_3d_entity);
        };
        vertex_3d.set_vec3(&new_position);
    }

    vertex_manager.on_vertex_3d_moved(
        &client,
        &face_manager,
        &mut meshes,
        &mesh_handle_q,
        &face_3d_q,
        &mut transform_q,
        &vertex_3d_entity,
    );

    canvas.queue_resync_shapes();

    system_state.apply(world);

    return vec![ShapeAction::MoveVertex(
        vertex_2d_entity,
        new_position,
        old_position,
        false,
    )];
}
