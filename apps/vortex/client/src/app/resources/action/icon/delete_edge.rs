use bevy_ecs::{
    prelude::{Commands, Query, World},
    system::SystemState,
};
use bevy_log::info;

use naia_bevy_client::Client;

use vortex_proto::components::IconEdge;

use crate::app::resources::{
    action::icon::{
        select_shape::{entity_request_release, select_shape},
        IconAction,
    },
    icon_manager::IconManager,
    shape_data::CanvasShape,
};

pub(crate) fn execute(
    world: &mut World,
    icon_manager: &mut IconManager,
    action: IconAction,
) -> Vec<IconAction> {
    let IconAction::DeleteEdge(edge_entity, shape_to_select_opt) = action else {
        panic!("Expected DeleteEdge");
    };

    info!("DeleteEdge(edge_entity: `{:?}`)", edge_entity);
    let mut system_state: SystemState<(Commands, Client, Query<&IconEdge>)> =
        SystemState::new(world);
    let (mut commands, mut client, edge_q) = system_state.get_mut(world);

    let edge = edge_q.get(edge_entity).unwrap();
    let vertex_start = edge.start.get(&client).unwrap();
    let vertex_end = edge.end.get(&client).unwrap();

    // delete edge
    commands.entity(edge_entity).despawn();

    // store vertices that will make a new face
    let mut deleted_face_vertex_entities = Vec::new();
    if let Some(connected_face_keys) = icon_manager.edge_connected_faces(&edge_entity) {
        for face_key in connected_face_keys {
            let local_face_entity = icon_manager
                .local_face_entity_from_face_key(&face_key)
                .unwrap();

            let face_has_net_entity = icon_manager
                .net_face_entity_from_face_key(&face_key)
                .is_some();

            let mut vertices = vec![face_key.vertex_a, face_key.vertex_b, face_key.vertex_c];
            vertices.retain(|vertex| *vertex != vertex_start && *vertex != vertex_end);
            if vertices.len() != 1 {
                panic!("expected 1 vertices, got {}!", vertices.len());
            }
            let face_vertex = vertices[0];

            deleted_face_vertex_entities.push((
                face_vertex,
                local_face_entity,
                face_has_net_entity,
            ));
        }
    }
    let deleted_face_vertex_entities_opt = if deleted_face_vertex_entities.len() > 0 {
        Some(deleted_face_vertex_entities)
    } else {
        None
    };

    // cleanup mappings
    icon_manager.cleanup_deleted_edge(&mut commands, &edge_entity);

    icon_manager.deselect_shape();

    // select entities as needed
    if let Some((shape_to_select, shape_type)) = shape_to_select_opt {
        let entity_to_request = select_shape(icon_manager, Some((shape_to_select, shape_type)));
        entity_request_release(&mut commands, &mut client, entity_to_request, None);
    }

    system_state.apply(world);

    return vec![IconAction::CreateEdge(
        vertex_start,
        vertex_end,
        (edge_entity, CanvasShape::Edge),
        deleted_face_vertex_entities_opt,
        Some(edge_entity),
    )];
}
