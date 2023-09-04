use bevy_ecs::{
    prelude::{Commands, Entity, Query, World},
    system::{ResMut, SystemState},
};
use bevy_log::info;

use naia_bevy_client::{Client, CommandsExt, ReplicationConfig};

use math::Vec3;
use render_api::{
    base::{CpuMaterial, CpuMesh},
    components::Transform,
    Assets,
};

use vortex_proto::components::{FileType, FileTypeValue, OwnedByFile, Vertex3d};

use crate::app::{
    components::{VertexTypeData, Vertex2d, VertexEntry},
    resources::{
        action::{create_edge::create_networked_edge, select_shape::deselect_all_selected_shapes, ShapeAction},
        action_stack::ActionStack,
        camera_manager::CameraManager,
        shape_manager::FaceKey,
        shape_manager::{CanvasShape, ShapeManager},
    },
};

pub(crate) fn execute(
    world: &mut World,
    action_stack: &mut ActionStack<ShapeAction>,
    tab_file_entity: &Entity,
    vertex_type_data: VertexTypeData,
    position: Vec3,
    old_vertex_entities_opt: Option<(Entity, Entity)>,
) -> Vec<ShapeAction> {
    let mut entities_to_release = Vec::new();
    let deselected_vertex_2d_entity_store;
    let selected_vertex_3d;
    let selected_vertex_2d;

    {
        match &vertex_type_data {
            VertexTypeData::Skel(parent_entity, _) => {
                info!(
                    "CreateVertexSkel(parent: {:?}, position: {:?})",
                    parent_entity, position
                );
            }
            VertexTypeData::Mesh(_, _) => {
                info!("CreateVertexMesh(position: {:?})", position);
            }
        }
    }

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
    let (deselected_vertex_2d_entity, vertex_3d_entity_to_release) = deselect_all_selected_shapes(&mut shape_manager);
    deselected_vertex_2d_entity_store = deselected_vertex_2d_entity;
    if let Some(entity) = vertex_3d_entity_to_release {
        let mut entity_mut = commands.entity(entity);
        if entity_mut.authority(&client).is_some() {
            entity_mut.release_authority(&mut client);
        }
    }

    let file_type_value = vertex_type_data.to_file_type_value();

    // create vertex
    let (new_vertex_2d_entity, new_vertex_3d_entity) = create_networked_vertex(
        &mut commands,
        &mut client,
        &mut camera_manager,
        &mut shape_manager,
        &mut meshes,
        &mut materials,
        position,
        *tab_file_entity,
        file_type_value,
        &mut entities_to_release,
    );

    // migrate undo entities
    if let Some((old_vertex_2d_entity, old_vertex_3d_entity)) = old_vertex_entities_opt {
        action_stack.migrate_vertex_entities(
            old_vertex_2d_entity,
            new_vertex_2d_entity,
            old_vertex_3d_entity,
            new_vertex_3d_entity,
        );
    }

    system_state.apply(world);

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

    match vertex_type_data {
        VertexTypeData::Skel(parent_vertex_2d_entity, children_opt) => {
            if let Some(children) = children_opt {
                create_networked_children_tree(
                    action_stack,
                    &mut commands,
                    &mut client,
                    &mut camera_manager,
                    &mut shape_manager,
                    &mut meshes,
                    &mut materials,
                    new_vertex_2d_entity,
                    children,
                    *tab_file_entity,
                    &mut entities_to_release,
                );
            }
            create_networked_edge(
                &mut commands,
                &mut client,
                &mut camera_manager,
                &mut shape_manager,
                &mut meshes,
                &mut materials,
                parent_vertex_2d_entity,
                new_vertex_2d_entity,
                new_vertex_3d_entity,
                *tab_file_entity,
                FileTypeValue::Skel,
                &mut entities_to_release,
            );
        }
        VertexTypeData::Mesh(connected_vertex_entities, connected_face_vertex_pairs) => {
            let mut edge_3d_entities = Vec::new();
            for (connected_vertex_entity, old_edge_opt) in connected_vertex_entities {
                let (new_edge_2d_entity, new_edge_3d_entity) = create_networked_edge(
                    &mut commands,
                    &mut client,
                    &mut camera_manager,
                    &mut shape_manager,
                    &mut meshes,
                    &mut materials,
                    connected_vertex_entity,
                    new_vertex_2d_entity,
                    new_vertex_3d_entity,
                    *tab_file_entity,
                    FileTypeValue::Mesh,
                    &mut entities_to_release,
                );
                edge_3d_entities.push(new_edge_3d_entity);
                if let Some(old_edge_2d_entity) = old_edge_opt {
                    action_stack.migrate_edge_entities(old_edge_2d_entity, new_edge_2d_entity);
                }
            }
            for (
                connected_face_vertex_a_2d,
                connected_face_vertex_b_2d,
                old_face_2d_entity,
                create_face_3d,
            ) in connected_face_vertex_pairs
            {
                let connected_face_vertex_a_3d = shape_manager
                    .vertex_entity_2d_to_3d(&connected_face_vertex_a_2d)
                    .unwrap();
                let connected_face_vertex_b_3d = shape_manager
                    .vertex_entity_2d_to_3d(&connected_face_vertex_b_2d)
                    .unwrap();
                let face_key = FaceKey::new(
                    new_vertex_3d_entity,
                    connected_face_vertex_a_3d,
                    connected_face_vertex_b_3d,
                );

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
        }
    };

    // select vertex
    shape_manager.select_shape(&new_vertex_2d_entity, CanvasShape::Vertex);
    selected_vertex_3d = new_vertex_3d_entity;
    selected_vertex_2d = new_vertex_2d_entity;

    system_state.apply(world);

    // release all non-selected vertices
    {
        let mut system_state: SystemState<(Commands, Client)> = SystemState::new(world);
        let (mut commands, mut client) = system_state.get_mut(world);

        for entity_to_release in entities_to_release {
            if entity_to_release != selected_vertex_3d {
                commands
                    .entity(entity_to_release)
                    .release_authority(&mut client);
            }
        }

        system_state.apply(world);
    }

    return vec![ShapeAction::DeleteVertex(
        selected_vertex_2d,
        deselected_vertex_2d_entity_store,
    )];
}

pub fn create_networked_vertex(
    commands: &mut Commands,
    client: &mut Client,
    camera_manager: &mut CameraManager,
    shape_manager: &mut ShapeManager,
    meshes: &mut Assets<CpuMesh>,
    materials: &mut Assets<CpuMaterial>,
    position: Vec3,
    file_entity: Entity,
    file_type: FileTypeValue,
    entities_to_release: &mut Vec<Entity>,
) -> (Entity, Entity) {
    // create new 3d vertex
    let mut owned_by_file_component = OwnedByFile::new();
    owned_by_file_component
        .file_entity
        .set(client, &file_entity);
    let new_vertex_3d_entity = commands
        .spawn_empty()
        .enable_replication(client)
        .configure_replication(ReplicationConfig::Delegated)
        .insert(Vertex3d::from_vec3(position))
        .insert(owned_by_file_component)
        .insert(FileType::new(file_type))
        .id();

    entities_to_release.push(new_vertex_3d_entity);

    // create new 2d vertex, add local components to 3d vertex
    let new_vertex_2d_entity = shape_manager.vertex_3d_postprocess(
        commands,
        meshes,
        materials,
        camera_manager,
        new_vertex_3d_entity,
        false,
        Some(file_entity),
        Vertex2d::CHILD_COLOR,
    );

    return (new_vertex_2d_entity, new_vertex_3d_entity);
}

pub(crate) fn create_networked_children_tree(
    action_stack: &mut ActionStack<ShapeAction>,
    commands: &mut Commands,
    client: &mut Client,
    camera_manager: &mut CameraManager,
    shape_manager: &mut ShapeManager,
    meshes: &mut Assets<CpuMesh>,
    materials: &mut Assets<CpuMaterial>,
    parent_vertex_2d_entity: Entity,
    children: Vec<VertexEntry>,
    file_entity: Entity,
    entities_to_release: &mut Vec<Entity>,
) {
    for child in children {
        let position = child.position();
        let grandchildren_opt = child.children();
        let old_child_vertex_3d_entity = child.entity_3d();
        let old_child_vertex_2d_entity = child.entity_2d();

        let (new_child_vertex_2d_entity, new_child_vertex_3d_entity) = create_networked_vertex(
                commands,
                client,
                camera_manager,
                shape_manager,
                meshes,
                materials,
                position,
                file_entity,
                FileTypeValue::Skel,
                entities_to_release,
            );
        action_stack.migrate_vertex_entities(
            old_child_vertex_2d_entity,
            new_child_vertex_2d_entity,
            old_child_vertex_3d_entity,
            new_child_vertex_3d_entity,
        );
        create_networked_edge(
            commands,
            client,
            camera_manager,
            shape_manager,
            meshes,
            materials,
            parent_vertex_2d_entity,
            new_child_vertex_2d_entity,
            new_child_vertex_3d_entity,
            file_entity,
            FileTypeValue::Skel,
            entities_to_release,
        );
        if let Some(grandchildren) = grandchildren_opt {
            create_networked_children_tree(
                action_stack,
                commands,
                client,
                camera_manager,
                shape_manager,
                meshes,
                materials,
                new_child_vertex_2d_entity,
                grandchildren,
                file_entity,
                entities_to_release,
            );
        }
    }
}