use bevy_ecs::{
    prelude::{Commands, Entity, Query, World},
    system::{ResMut, SystemState},
};
use bevy_log::info;

use naia_bevy_client::{Client, CommandsExt};

use vortex_proto::components::Edge3d;

use crate::app::resources::{
    action::Action,
    action_stack::ActionStack,
    shape_manager::{CanvasShape, ShapeManager},
};

pub(crate) fn execute(
    world: &mut World,
    edge_2d_entity: Entity,
    shape_2d_to_select_opt: Option<(Entity, CanvasShape)>,
) -> Vec<Action> {
    info!("DeleteEdge(edge_2d_entity: `{:?}`)", edge_2d_entity);
    let mut system_state: SystemState<(Commands, Client, ResMut<ShapeManager>, Query<&Edge3d>)> =
        SystemState::new(world);
    let (mut commands, mut client, mut shape_manager, edge_3d_q) = system_state.get_mut(world);

    let Some(edge_3d_entity_ref) = shape_manager.edge_entity_2d_to_3d(&edge_2d_entity) else {
        panic!("failed to get edge 3d entity for edge 2d entity `{:?}`!", edge_2d_entity)
    };
    let edge_3d_entity = edge_3d_entity_ref;

    let edge_3d = edge_3d_q.get(edge_3d_entity).unwrap();
    let vertex_start_3d = edge_3d.start.get(&client).unwrap();
    let vertex_end_3d = edge_3d.end.get(&client).unwrap();
    let vertex_start_2d = shape_manager
        .vertex_entity_3d_to_2d(&vertex_start_3d)
        .unwrap();
    let vertex_end_2d = shape_manager
        .vertex_entity_3d_to_2d(&vertex_end_3d)
        .unwrap();

    // delete 3d edge
    commands.entity(edge_3d_entity).despawn();

    // store vertices that will make a new face
    let mut deleted_face_vertex_2d_entities = Vec::new();
    if let Some(connected_face_keys) = shape_manager.edge_connected_faces(&edge_3d_entity) {
        for face_key in connected_face_keys {
            let face_2d_entity = shape_manager
                .face_2d_entity_from_face_key(&face_key)
                .unwrap();

            let face_has_3d_entity = shape_manager
                .face_3d_entity_from_face_key(&face_key)
                .is_some();

            let mut vertices_3d = vec![
                face_key.vertex_3d_a,
                face_key.vertex_3d_b,
                face_key.vertex_3d_c,
            ];
            vertices_3d.retain(|vertex| *vertex != vertex_start_3d && *vertex != vertex_end_3d);
            if vertices_3d.len() != 1 {
                panic!("expected 1 vertices, got {}!", vertices_3d.len());
            }
            let face_vertex_3d = vertices_3d[0];
            let face_vertex_2d_entity = shape_manager
                .vertex_entity_3d_to_2d(&face_vertex_3d)
                .unwrap();

            deleted_face_vertex_2d_entities.push((
                face_vertex_2d_entity,
                face_2d_entity,
                face_has_3d_entity,
            ));
        }
    }
    let deleted_face_vertex_2d_entities_opt = if deleted_face_vertex_2d_entities.len() > 0 {
        Some(deleted_face_vertex_2d_entities)
    } else {
        None
    };

    // cleanup mappings
    shape_manager.cleanup_deleted_edge(&mut commands, &edge_3d_entity);

    // select entities as needed
    if let Some((shape_2d_to_select, shape_type)) = shape_2d_to_select_opt {
        if let Some(shape_3d_entity_to_request) =
            ActionStack::select_shape(&mut shape_manager, Some((shape_2d_to_select, shape_type)))
        {
            //info!("request_entities({:?})", shape_3d_entity_to_request);
            let mut entity_mut = commands.entity(shape_3d_entity_to_request);
            if entity_mut.authority(&client).is_some() {
                entity_mut.request_authority(&mut client);
            }
        }
    } else {
        shape_manager.deselect_shape();
    }

    system_state.apply(world);

    return vec![Action::CreateEdge(
        vertex_start_2d,
        vertex_end_2d,
        (edge_2d_entity, CanvasShape::Edge),
        deleted_face_vertex_2d_entities_opt,
        Some(edge_2d_entity),
    )];
}