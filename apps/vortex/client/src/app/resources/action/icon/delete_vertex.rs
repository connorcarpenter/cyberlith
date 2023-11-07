use bevy_ecs::{
    prelude::{Commands, Entity, Query, World},
    system::SystemState,
};
use bevy_log::info;

use naia_bevy_client::Client;

use vortex_proto::components::{IconEdge, IconVertex};

use crate::app::{
    components::IconVertexActionData,
    resources::{
        action::icon::{
            select_shape::{entity_request_release, select_shape},
            IconAction,
        },
        icon_data::IconFaceKey,
        icon_manager::IconManager,
        shape_data::CanvasShape,
    },
};

pub(crate) fn execute(
    world: &mut World,
    icon_manager: &mut IconManager,
    action: IconAction,
) -> Vec<IconAction> {
    let IconAction::DeleteVertex(vertex_entity, vertex_to_select_opt) = action else {
        panic!("Expected DeleteVertex");
    };

    info!("DeleteVertex({:?})", vertex_entity);

    let mut system_state: SystemState<(
        Commands,
        Client,
        Query<(Entity, &IconVertex)>,
        Query<&IconEdge>,
    )> = SystemState::new(world);
    let (mut commands, mut client, vertex_q, edge_q) = system_state.get_mut(world);

    let mut connected_vertices_entities = Vec::new();
    let mut connected_face_vertex_entities = Vec::new();

    let Some(connected_edges) = icon_manager.vertex_get_edges(&vertex_entity) else {
        panic!("Failed to get connected edges for vertex entity {:?}!", vertex_entity);
    };
    let connected_edges = connected_edges.iter().map(|edge| *edge).collect::<Vec<_>>();
    for edge_entity in connected_edges {
        let Ok(edge) = edge_q.get(edge_entity) else {
            panic!("Failed to get IconEdge for edge entity {:?}!", edge_entity);
        };
        let start_vertex_entity = edge.start.get(&client).unwrap();
        let end_vertex_entity = edge.end.get(&client).unwrap();

        let connected_vertex_entity = if start_vertex_entity == vertex_entity {
            end_vertex_entity
        } else {
            start_vertex_entity
        };

        connected_vertices_entities.push((connected_vertex_entity, Some(edge_entity)));
    }
    let Some(connected_faces) = icon_manager.vertex_get_faces(&vertex_entity) else {
        panic!("Failed to get connected faces for vertex entity {:?}!", vertex_entity);
    };
    let connected_faces: Vec<IconFaceKey> = connected_faces.iter().map(|face| *face).collect();
    for face_key in connected_faces {
        let face_net_entity_exists = icon_manager
            .net_face_entity_from_face_key(&face_key)
            .is_some();

        let mut vertices = vec![face_key.vertex_a, face_key.vertex_b, face_key.vertex_c];
        vertices.retain(|vertex| *vertex != vertex_entity);

        let face_local_entity = icon_manager
            .local_face_entity_from_face_key(&face_key)
            .unwrap();

        connected_face_vertex_entities.push((
            vertices[0],
            vertices[1],
            face_local_entity,
            face_net_entity_exists,
        ));
    }

    let rev_vertex_type_data =
        IconVertexActionData::new(connected_vertices_entities, connected_face_vertex_entities);

    let Ok((_, vertex)) = vertex_q.get(vertex_entity) else {
        panic!("Failed to get IconVertex for vertex entity {:?}!", vertex_entity);
    };
    let vertex_position = vertex.as_vec2();

    handle_vertex_despawn(
        &mut commands,
        &mut client,
        icon_manager,
        vertex_entity,
        vertex_to_select_opt,
    );

    system_state.apply(world);

    return vec![IconAction::CreateVertex(
        rev_vertex_type_data,
        vertex_position,
        Some(vertex_entity),
    )];
}

fn handle_vertex_despawn(
    commands: &mut Commands,
    client: &mut Client,
    icon_manager: &mut IconManager,
    vertex_entity: Entity,
    vertex_to_select_opt: Option<(Entity, CanvasShape)>,
) {
    // delete vertex
    commands.entity(vertex_entity).despawn();

    // cleanup mappings
    icon_manager.cleanup_deleted_vertex(&vertex_entity);

    icon_manager.deselect_shape();

    // select entities as needed
    if let Some((vertex_to_select, vertex_type)) = vertex_to_select_opt {
        let entity_to_request = select_shape(icon_manager, Some((vertex_to_select, vertex_type)));
        entity_request_release(commands, client, entity_to_request, None);
    }
}
