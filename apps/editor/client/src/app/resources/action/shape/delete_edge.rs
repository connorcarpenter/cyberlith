use bevy_ecs::{
    prelude::{Commands, Query, World},
    system::{ResMut, SystemState},
};
use bevy_log::info;

use naia_bevy_client::Client;

use vortex_proto::components::Edge3d;

use crate::app::{plugin::Main, resources::{
    action::shape::{
        select_shape::{entity_request_release, select_shape},
        ShapeAction,
    },
    canvas::Canvas,
    edge_manager::EdgeManager,
    face_manager::FaceManager,
    input::InputManager,
    shape_data::CanvasShape,
    vertex_manager::VertexManager,
}};

pub(crate) fn execute(
    world: &mut World,
    input_manager: &mut InputManager,
    action: ShapeAction,
) -> Vec<ShapeAction> {
    let ShapeAction::DeleteEdge(edge_2d_entity, shape_2d_to_select_opt) = action else {
        panic!("Expected DeleteEdge");
    };

    info!("DeleteEdge(edge_2d_entity: `{:?}`)", edge_2d_entity);
    let mut system_state: SystemState<(
        Commands,
        Client<Main>,
        ResMut<Canvas>,
        ResMut<VertexManager>,
        ResMut<EdgeManager>,
        ResMut<FaceManager>,
        Query<&Edge3d>,
    )> = SystemState::new(world);
    let (
        mut commands,
        mut client,
        mut canvas,
        mut vertex_manager,
        mut edge_manager,
        mut face_manager,
        edge_3d_q,
    ) = system_state.get_mut(world);

    let Some(edge_3d_entity_ref) = edge_manager.edge_entity_2d_to_3d(&edge_2d_entity) else {
        panic!("failed to get edge 3d entity for edge 2d entity `{:?}`!", edge_2d_entity)
    };
    let edge_3d_entity = edge_3d_entity_ref;

    let edge_3d = edge_3d_q.get(edge_3d_entity).unwrap();
    let vertex_start_3d = edge_3d.start.get(&client).unwrap();
    let vertex_end_3d = edge_3d.end.get(&client).unwrap();
    let vertex_start_2d = vertex_manager
        .vertex_entity_3d_to_2d(&vertex_start_3d)
        .unwrap();
    let vertex_end_2d = vertex_manager
        .vertex_entity_3d_to_2d(&vertex_end_3d)
        .unwrap();

    // delete 3d edge
    commands.entity(edge_3d_entity).despawn();

    // store vertices that will make a new face
    let mut deleted_face_vertex_2d_entities = Vec::new();
    if let Some(connected_face_keys) = edge_manager.edge_connected_faces(&edge_3d_entity) {
        for face_key in connected_face_keys {
            let face_2d_entity = face_manager
                .face_2d_entity_from_face_key(&face_key)
                .unwrap();

            let face_has_3d_entity = face_manager
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
            let face_vertex_2d_entity = vertex_manager
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
    edge_manager.cleanup_deleted_edge(
        &mut commands,
        &mut canvas,
        input_manager,
        &mut vertex_manager,
        Some(&mut face_manager),
        &edge_3d_entity,
    );

    input_manager.deselect_shape(&mut canvas);

    // select entities as needed
    if let Some((shape_2d_to_select, shape_type)) = shape_2d_to_select_opt {
        let entity_to_request = select_shape(
            &mut canvas,
            input_manager,
            &vertex_manager,
            &edge_manager,
            &face_manager,
            Some((shape_2d_to_select, shape_type)),
        );
        entity_request_release(&mut commands, &mut client, entity_to_request, None);
    }

    system_state.apply(world);

    return vec![ShapeAction::CreateEdge(
        vertex_start_2d,
        vertex_end_2d,
        (edge_2d_entity, CanvasShape::Edge),
        deleted_face_vertex_2d_entities_opt,
        Some(edge_2d_entity),
    )];
}
