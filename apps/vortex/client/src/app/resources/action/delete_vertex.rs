use bevy_ecs::{
    prelude::{Commands, Entity, Query, World},
    system::{ResMut, SystemState},
};
use bevy_log::info;

use naia_bevy_client::Client;

use vortex_proto::{
    components::{
        Edge3d,
        FileType, FileTypeValue, Vertex3d,
    },
};

use crate::app::{
    components::VertexTypeData,
    resources::{
        action::Action,
        action_stack::ActionStack,
        shape_manager::{CanvasShape, ShapeManager},
    },
};

pub(crate) fn execute(world: &mut World, vertex_2d_entity: Entity, vertex_2d_to_select_opt: Option<(Entity, CanvasShape)>) -> Vec<Action> {
    info!("DeleteVertex({:?})", vertex_2d_entity);

    let mut system_state: SystemState<(
        Commands,
        Client,
        ResMut<ShapeManager>,
        Query<(Entity, &Vertex3d)>,
        Query<&Edge3d>,
        Query<&FileType>,
    )> = SystemState::new(world);
    let (mut commands, mut client, mut shape_manager, vertex_q, edge_3d_q, file_type_q) =
        system_state.get_mut(world);

    let vertex_3d_entity = shape_manager
        .vertex_entity_2d_to_3d(&vertex_2d_entity)
        .unwrap();

    let Ok(file_type) = file_type_q.get(vertex_3d_entity) else {
        panic!("Failed to get FileType for vertex entity {:?}!", vertex_3d_entity);
    };
    let file_type_value = *file_type.value;

    match file_type_value {
        FileTypeValue::Skel => {
            // get parent entity
            let parent_vertex_2d_entity = {
                let mut parent_vertex_3d_entity = None;
                for edge_3d in edge_3d_q.iter() {
                    let Some(child_entity) = edge_3d.end.get(&client) else {
                        continue;
                    };
                    let Some(parent_entity) = edge_3d.start.get(&client) else {
                        continue;
                    };
                    if child_entity == vertex_3d_entity {
                        parent_vertex_3d_entity = Some(parent_entity);
                        break;
                    }
                }
                if parent_vertex_3d_entity.is_none() {
                    panic!(
                        "Failed to find parent vertex for vertex entity {:?}!",
                        vertex_3d_entity
                    );
                }
                shape_manager
                    .vertex_entity_3d_to_2d(&parent_vertex_3d_entity.unwrap())
                    .unwrap()
            };

            // get entries
            let entry_contents_opt = {
                let entries = ActionStack::convert_vertices_to_tree(
                    &client,
                    &mut shape_manager,
                    &vertex_3d_entity,
                    &vertex_q,
                    &edge_3d_q,
                );

                Some(entries)
            };

            let rev_vertex_type_data = VertexTypeData::Skel(
                parent_vertex_2d_entity,
                entry_contents_opt.map(|entries| {
                    entries.into_iter().map(|(_, entry)| entry).collect()
                }),
            );

            let Ok((_, vertex_3d)) = vertex_q.get(vertex_3d_entity) else {
                panic!("Failed to get VertexChild for vertex entity {:?}!", vertex_3d_entity);
            };
            let vertex_3d_position = vertex_3d.as_vec3();

            ActionStack::handle_common_vertex_despawn(
                &mut commands,
                &mut client,
                &mut shape_manager,
                vertex_3d_entity,
                vertex_2d_to_select_opt,
            );

            system_state.apply(world);

            return vec![Action::CreateVertex(
                rev_vertex_type_data,
                vertex_3d_position,
                Some((vertex_2d_entity, vertex_3d_entity)),
            )];
        }
        FileTypeValue::Mesh => {
            let mut connected_vertices_2d_entities = Vec::new();
            let mut connected_face_vertex_2d_entities = Vec::new();

            let Some(connected_edges) = shape_manager.vertex_connected_edges(&vertex_3d_entity) else {
                panic!("Failed to get connected edges for vertex entity {:?}!", vertex_3d_entity);
            };
            for edge_3d_entity in connected_edges {
                let edge_3d = edge_3d_q.get(edge_3d_entity).unwrap();
                let start_vertex_3d_entity = edge_3d.start.get(&client).unwrap();
                let end_vertex_3d_entity = edge_3d.end.get(&client).unwrap();

                let connected_vertex_3d_entity = if start_vertex_3d_entity == vertex_3d_entity {
                    end_vertex_3d_entity
                } else {
                    start_vertex_3d_entity
                };

                let Some(connected_vertex_2d_entity) = shape_manager.vertex_entity_3d_to_2d(&connected_vertex_3d_entity) else {
                    panic!("Failed to get connected vertex 2d entity for vertex entity {:?}!", connected_vertex_3d_entity);
                };

                let edge_2d_entity = shape_manager.edge_entity_3d_to_2d(&edge_3d_entity).unwrap();

                connected_vertices_2d_entities.push((connected_vertex_2d_entity, Some(edge_2d_entity)));
            }
            let Some(connected_faces) = shape_manager.vertex_connected_faces(&vertex_3d_entity) else {
                panic!("Failed to get connected faces for vertex entity {:?}!", vertex_3d_entity);
            };
            for face_key in connected_faces {

                let face_3d_entity_exists = shape_manager.face_3d_entity_from_face_key(&face_key).is_some();

                let mut vertices_3d = vec![face_key.vertex_3d_a, face_key.vertex_3d_b, face_key.vertex_3d_c];
                vertices_3d.retain(|vertex| *vertex != vertex_3d_entity);
                let vertices_2d: Vec<Entity> = vertices_3d.iter().map(|vertex| shape_manager.vertex_entity_3d_to_2d(&vertex).unwrap()).collect();

                let face_2d_entity = shape_manager.face_2d_entity_from_face_key(&face_key).unwrap();

                connected_face_vertex_2d_entities.push((vertices_2d[0], vertices_2d[1], face_2d_entity, face_3d_entity_exists));
            }

            let rev_vertex_type_data =
                VertexTypeData::Mesh(connected_vertices_2d_entities, connected_face_vertex_2d_entities);

            let Ok((_, vertex_3d)) = vertex_q.get(vertex_3d_entity) else {
                panic!("Failed to get Vertex3d for vertex entity {:?}!", vertex_3d_entity);
            };
            let vertex_3d_position = vertex_3d.as_vec3();

            ActionStack::handle_common_vertex_despawn(
                &mut commands,
                &mut client,
                &mut shape_manager,
                vertex_3d_entity,
                vertex_2d_to_select_opt,
            );

            system_state.apply(world);

            return vec![Action::CreateVertex(
                rev_vertex_type_data,
                vertex_3d_position,
                Some((vertex_2d_entity, vertex_3d_entity)),
            )];
        }
    }
}