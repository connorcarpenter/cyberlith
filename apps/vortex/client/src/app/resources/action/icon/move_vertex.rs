use bevy_ecs::{
    prelude::{Query, World},
    system::{ResMut, SystemState},
};
use bevy_log::info;

use naia_bevy_client::Client;

use render_api::{base::CpuMesh, components::Transform, Assets, Handle};

use vortex_proto::components::{IconFace, IconVertex};

use crate::app::resources::{action::icon::IconAction, canvas::Canvas, icon_manager::IconManager};

pub(crate) fn execute(world: &mut World, action: IconAction) -> Vec<IconAction> {
    let IconAction::MoveVertex(vertex_entity, old_position, new_position, already_moved) = action else {
        panic!("Expected MoveVertex");
    };

    info!("MoveVertex({:?}, _, _, {})", vertex_entity, already_moved);
    let mut system_state: SystemState<(
        Client,
        ResMut<Canvas>,
        ResMut<Assets<CpuMesh>>,
        ResMut<IconManager>,
        Query<&Handle<CpuMesh>>,
        Query<&IconFace>,
        Query<&mut Transform>,
        Query<&mut IconVertex>,
    )> = SystemState::new(world);
    let (
        client,
        mut canvas,
        mut meshes,
        icon_manager,
        mesh_handle_q,
        face_q,
        mut transform_q,
        mut vertex_q,
    ) = system_state.get_mut(world);

    if !already_moved {
        // MoveVertex action happens after the vertex has already been moved, so we wouldn't need to do anything here ..
        // BUT we do need to update the vertex's position here in order to apply when undo/redo is executed
        let Ok(mut vertex) = vertex_q.get_mut(vertex_entity) else {
            panic!("Failed to get IconVertex for vertex entity {:?}!", vertex_entity);
        };
        vertex.set_vec2(&new_position);
    }

    icon_manager.on_vertex_moved(
        &client,
        &mut meshes,
        &mesh_handle_q,
        &face_q,
        &mut transform_q,
        &vertex_entity,
    );

    canvas.queue_resync_shapes();

    system_state.apply(world);

    return vec![IconAction::MoveVertex(
        vertex_entity,
        new_position,
        old_position,
        false,
    )];
}
