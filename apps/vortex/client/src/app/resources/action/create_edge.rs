use bevy_ecs::{
    prelude::{Commands, Entity, Query, World},
    system::{ResMut, SystemState},
};
use bevy_log::info;

use naia_bevy_client::{Client, CommandsExt, ReplicationConfig};

use render_api::{
    base::{CpuMaterial, CpuMesh},
    components::Transform,
    Assets,
};

use vortex_proto::components::{Edge3d, EdgeAngle, FileType, FileTypeValue, OwnedByFile};

use crate::app::{
    components::Vertex2d,
    resources::{
        face_manager::FaceManager,
        vertex_manager::VertexManager,
        action::{select_shape::deselect_all_selected_shapes, ActionStack, ShapeAction},
        camera_manager::CameraManager,
        shape_manager::ShapeManager,
        shape_data::{CanvasShape, FaceKey},
    },
};
use crate::app::resources::edge_manager::EdgeManager;

pub(crate) fn execute(
    world: &mut World,
    action_stack: &mut ActionStack<ShapeAction>,
    tab_file_entity: &Entity,
    vertex_2d_entity_a: Entity,
    vertex_2d_entity_b: Entity,
    shape_to_select: (Entity, CanvasShape),
    face_to_create_opt: Option<Vec<(Entity, Entity, bool)>>,
    old_edge_entities_opt: Option<Entity>,
) -> Vec<ShapeAction> {
    let (mut shape_2d_entity_to_select, shape_2d_type_to_select) = shape_to_select;

    info!(
        "CreateEdge(vertex_a: {:?}, vertex_b: {:?}, (shape_2d_entity_to_select: {:?}, {:?}), face_to_create_opt: {:?})",
        vertex_2d_entity_a, vertex_2d_entity_b, shape_2d_entity_to_select, shape_2d_type_to_select, face_to_create_opt
    );

    let mut entities_to_release = Vec::new();
    let selected_shape_3d;
    let deselected_shape_2d_entity_store;
    let created_edge_2d_entity;
    let mut edge_3d_entities = Vec::new();

    {
        let mut system_state: SystemState<(
            Commands,
            Client,
            ResMut<CameraManager>,
            ResMut<ShapeManager>,
            ResMut<VertexManager>,
            ResMut<EdgeManager>,
            ResMut<FaceManager>,
            ResMut<Assets<CpuMesh>>,
            ResMut<Assets<CpuMaterial>>,
        )> = SystemState::new(world);
        let (
            mut commands,
            mut client,
            mut camera_manager,
            mut shape_manager,
            mut vertex_manager,
            mut edge_manager,
            mut face_manager,
            mut meshes,
            mut materials,
        ) = system_state.get_mut(world);

        // deselect all selected vertices
        let (deselected_shape_2d_entity, shape_3d_entity_to_release) =
            deselect_all_selected_shapes(&mut shape_manager, &vertex_manager, &edge_manager, &face_manager);
        deselected_shape_2d_entity_store = deselected_shape_2d_entity;
        if let Some(entity) = shape_3d_entity_to_release {
            let mut entity_mut = commands.entity(entity);
            if entity_mut.authority(&client).is_some() {
                entity_mut.release_authority(&mut client);
            }
        }

        // get 3d version of first vertex
        let vertex_3d_entity_b = vertex_manager
            .vertex_entity_2d_to_3d(&vertex_2d_entity_b)
            .unwrap();

        // create edge
        let (new_edge_2d_entity, new_edge_3d_entity) = edge_manager.create_networked_edge(
            &mut commands,
            &mut client,
            &mut camera_manager,
            &mut vertex_manager,
            &mut face_manager,
            &mut meshes,
            &mut materials,
            vertex_2d_entity_a,
            vertex_2d_entity_b,
            vertex_3d_entity_b,
            *tab_file_entity,
            FileTypeValue::Mesh,
            None,
            &mut entities_to_release,
        );
        created_edge_2d_entity = new_edge_2d_entity;
        edge_3d_entities.push(new_edge_3d_entity);

        // migrate undo entities
        if let Some(old_edge_2d_entity) = old_edge_entities_opt {
            action_stack.migrate_edge_entities(old_edge_2d_entity, new_edge_2d_entity);
            if shape_2d_type_to_select == CanvasShape::Edge {
                if shape_2d_entity_to_select == old_edge_2d_entity {
                    shape_2d_entity_to_select = new_edge_2d_entity;
                }
            }
        }

        // select vertex
        shape_manager.select_shape(&shape_2d_entity_to_select, shape_2d_type_to_select);
        selected_shape_3d = ShapeManager::shape_entity_2d_to_3d(&vertex_manager, &edge_manager, &face_manager, &shape_2d_entity_to_select, shape_2d_type_to_select)
            .unwrap();

        system_state.apply(world);
    }

    // release all non-selected shapes
    {
        let mut system_state: SystemState<(Commands, Client)> = SystemState::new(world);
        let (mut commands, mut client) = system_state.get_mut(world);

        for entity_to_release in entities_to_release {
            if entity_to_release != selected_shape_3d {
                commands
                    .entity(entity_to_release)
                    .release_authority(&mut client);
            }
        }

        system_state.apply(world);
    }

    // create face if necessary
    {
        if let Some(vertex_2d_entities) = face_to_create_opt {
            let mut system_state: SystemState<(
                Commands,
                Client,
                ResMut<CameraManager>,
                ResMut<VertexManager>,
                ResMut<EdgeManager>,
                ResMut<FaceManager>,
                ResMut<Assets<CpuMesh>>,
                ResMut<Assets<CpuMaterial>>,
                Query<&Transform>,
            )> = SystemState::new(world);
            let (
                mut commands,
                mut client,
                mut camera_manager,
                mut vertex_manager,
                mut edge_manager,
                mut face_manager,
                mut meshes,
                mut materials,
                transform_q,
            ) = system_state.get_mut(world);

            for (vertex_2d_of_face_to_create, old_face_2d_entity, create_face_3d) in
                vertex_2d_entities
            {
                let vertex_3d_a = vertex_manager
                    .vertex_entity_2d_to_3d(&vertex_2d_entity_a)
                    .unwrap();
                let vertex_3d_b = vertex_manager
                    .vertex_entity_2d_to_3d(&vertex_2d_entity_b)
                    .unwrap();
                let vertex_3d_c = vertex_manager
                    .vertex_entity_2d_to_3d(&vertex_2d_of_face_to_create)
                    .unwrap();
                let face_key = FaceKey::new(vertex_3d_a, vertex_3d_b, vertex_3d_c);

                face_manager.remove_new_face_key(&face_key);
                let new_face_2d_entity = face_manager.process_new_face(
                    &mut commands,
                    &mut camera_manager,
                    &mut vertex_manager,
                    &mut edge_manager,
                    &mut meshes,
                    &mut materials,
                    *tab_file_entity,
                    &face_key,
                );
                action_stack.migrate_face_entities(old_face_2d_entity, new_face_2d_entity);
                if create_face_3d {
                    face_manager.create_networked_face(
                        &mut commands,
                        &mut client,
                        &mut meshes,
                        &mut materials,
                        &mut camera_manager,
                        &transform_q,
                        &face_key,
                        [
                            edge_3d_entities[0],
                            edge_3d_entities[1],
                            edge_3d_entities[2],
                        ],
                        *tab_file_entity,
                    );
                }
            }

            system_state.apply(world);
        }
    }

    return vec![
        ShapeAction::DeleteEdge(created_edge_2d_entity, deselected_shape_2d_entity_store),
        ShapeAction::SelectShape(Some((created_edge_2d_entity, CanvasShape::Edge))),
    ];
}
