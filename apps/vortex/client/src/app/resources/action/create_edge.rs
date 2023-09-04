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

use vortex_proto::components::{Edge3d, FileType, FileTypeValue, OwnedByFile};

use crate::app::{resources::{
    action::{ActionStack, select_shape::deselect_all_selected_shapes, ShapeAction},
    camera_manager::CameraManager,
    shape_manager::FaceKey,
    shape_manager::{CanvasShape, ShapeManager},
}, components::Vertex2d};

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
            ResMut<Assets<CpuMesh>>,
            ResMut<Assets<CpuMaterial>>,
        )> = SystemState::new(world);
        let (
            mut commands,
            mut client,
            mut camera_manager,
            mut shape_manager,
            mut meshes,
            mut materials,
        ) = system_state.get_mut(world);

        // deselect all selected vertices
        let (deselected_shape_2d_entity, shape_3d_entity_to_release) =
            deselect_all_selected_shapes(&mut shape_manager);
        deselected_shape_2d_entity_store = deselected_shape_2d_entity;
        if let Some(entity) = shape_3d_entity_to_release {
            let mut entity_mut = commands.entity(entity);
            if entity_mut.authority(&client).is_some() {
                entity_mut.release_authority(&mut client);
            }
        }

        // get 3d version of first vertex
        let vertex_3d_entity_b = shape_manager
            .vertex_entity_2d_to_3d(&vertex_2d_entity_b)
            .unwrap();

        // create edge
        let (new_edge_2d_entity, new_edge_3d_entity) = create_networked_edge(
            &mut commands,
            &mut client,
            &mut camera_manager,
            &mut shape_manager,
            &mut meshes,
            &mut materials,
            vertex_2d_entity_a,
            vertex_2d_entity_b,
            vertex_3d_entity_b,
            *tab_file_entity,
            FileTypeValue::Mesh,
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
        selected_shape_3d = shape_manager
            .shape_entity_2d_to_3d(&shape_2d_entity_to_select, shape_2d_type_to_select)
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
                ResMut<ShapeManager>,
                ResMut<Assets<CpuMesh>>,
                ResMut<Assets<CpuMaterial>>,
                Query<&Transform>,
            )> = SystemState::new(world);
            let (
                mut commands,
                mut client,
                mut camera_manager,
                mut shape_manager,
                mut meshes,
                mut materials,
                transform_q,
            ) = system_state.get_mut(world);

            for (vertex_2d_of_face_to_create, old_face_2d_entity, create_face_3d) in
                vertex_2d_entities
            {
                let vertex_3d_a = shape_manager
                    .vertex_entity_2d_to_3d(&vertex_2d_entity_a)
                    .unwrap();
                let vertex_3d_b = shape_manager
                    .vertex_entity_2d_to_3d(&vertex_2d_entity_b)
                    .unwrap();
                let vertex_3d_c = shape_manager
                    .vertex_entity_2d_to_3d(&vertex_2d_of_face_to_create)
                    .unwrap();
                let face_key = FaceKey::new(vertex_3d_a, vertex_3d_b, vertex_3d_c);

                shape_manager.remove_new_face_key(&face_key);
                let new_face_2d_entity = shape_manager.process_new_face(
                    &mut commands,
                    &mut camera_manager,
                    &mut meshes,
                    &mut materials,
                    *tab_file_entity,
                    &face_key,
                );
                action_stack.migrate_face_entities(old_face_2d_entity, new_face_2d_entity);
                if create_face_3d {
                    shape_manager.create_networked_face_inner(
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

// return (new edge 2d entity, new edge 3d entity)
pub fn create_networked_edge(
    commands: &mut Commands,
    client: &mut Client,
    camera_manager: &mut CameraManager,
    shape_manager: &mut ShapeManager,
    meshes: &mut Assets<CpuMesh>,
    materials: &mut Assets<CpuMaterial>,
    parent_vertex_2d_entity: Entity,
    child_vertex_2d_entity: Entity,
    child_vertex_3d_entity: Entity,
    file_entity: Entity,
    file_type: FileTypeValue,
    entities_to_release: &mut Vec<Entity>,
) -> (Entity, Entity) {
    // create new 3d edge
    let parent_vertex_3d_entity = shape_manager
        .vertex_entity_2d_to_3d(&parent_vertex_2d_entity)
        .unwrap();

    let mut new_edge_3d_component = Edge3d::new();
    new_edge_3d_component
        .start
        .set(client, &parent_vertex_3d_entity);
    new_edge_3d_component
        .end
        .set(client, &child_vertex_3d_entity);
    let mut owned_by_file_component = OwnedByFile::new();
    owned_by_file_component
        .file_entity
        .set(client, &file_entity);
    let new_edge_3d_entity = commands
        .spawn_empty()
        .enable_replication(client)
        .configure_replication(ReplicationConfig::Delegated)
        .insert(new_edge_3d_component)
        .insert(owned_by_file_component)
        .insert(FileType::new(file_type))
        .id();

    // create new 2d edge, add local components to 3d edge
    let new_edge_2d_entity = shape_manager.edge_3d_postprocess(
        commands,
        meshes,
        materials,
        camera_manager,
        new_edge_3d_entity,
        parent_vertex_2d_entity,
        parent_vertex_3d_entity,
        child_vertex_2d_entity,
        child_vertex_3d_entity,
        Some(file_entity),
        Vertex2d::CHILD_COLOR,
        file_type == FileTypeValue::Skel,
    );

    entities_to_release.push(new_edge_3d_entity);

    (new_edge_2d_entity, new_edge_3d_entity)
}
