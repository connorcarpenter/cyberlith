use bevy_ecs::{
    prelude::{Commands, Entity, Query, World},
    system::{ResMut, SystemState},
};
use logging::info;

use naia_bevy_client::{Client, CommandsExt};

use render_api::{
    base::{CpuMaterial, CpuMesh},
    components::Transform,
};
use storage::Storage;

use editor_proto::components::FileExtension;

use crate::app::{
    components::VertexTypeData,
    plugin::Main,
    resources::{
        action::{
            shape::{select_shape::deselect_selected_shape, ShapeAction},
            ActionStack,
        },
        camera_manager::CameraManager,
        canvas::Canvas,
        edge_manager::EdgeManager,
        face_manager::FaceManager,
        input::InputManager,
        shape_data::{CanvasShape, FaceKey},
        vertex_manager::VertexManager,
    },
};

pub(crate) fn execute(
    world: &mut World,
    input_manager: &mut InputManager,
    action_stack: &mut ActionStack<ShapeAction>,
    tab_file_entity: Entity,
    action: ShapeAction,
) -> Vec<ShapeAction> {
    let ShapeAction::CreateVertex(vertex_type_data, position, old_vertex_entities_opt) = action
    else {
        panic!("Expected CreateVertex");
    };

    let mut entities_to_release = Vec::new();
    let deselected_vertex_2d_entity_store;
    let selected_vertex_3d;
    let selected_vertex_2d;

    let mut system_state: SystemState<(
        Commands,
        Client<Main>,
        ResMut<Canvas>,
        ResMut<CameraManager>,
        ResMut<VertexManager>,
        ResMut<EdgeManager>,
        ResMut<FaceManager>,
        ResMut<Storage<CpuMesh>>,
        ResMut<Storage<CpuMaterial>>,
    )> = SystemState::new(world);
    let (
        mut commands,
        mut client,
        mut canvas,
        mut camera_manager,
        mut vertex_manager,
        edge_manager,
        face_manager,
        mut meshes,
        mut materials,
    ) = system_state.get_mut(world);

    // deselect all selected vertices
    let (deselected_vertex_2d_entity, vertex_3d_entity_to_release) = deselect_selected_shape(
        &mut canvas,
        input_manager,
        &vertex_manager,
        &edge_manager,
        &face_manager,
    );
    deselected_vertex_2d_entity_store = deselected_vertex_2d_entity;
    if let Some(entity) = vertex_3d_entity_to_release {
        let mut entity_mut = commands.entity(entity);
        if entity_mut.authority(&client).is_some() {
            entity_mut.release_authority(&mut client);
        }
    }

    let file_type_value = vertex_type_data.to_file_type_value();

    // create vertex
    let parent_vertex_3d_entity_opt = match &vertex_type_data {
        VertexTypeData::Skel(parent_vertex_2d_entity, edge_angle, _) => {
            info!(
                "CreateVertexSkel(parent: {:?}, edge_angle: {:?}, position: {:?})",
                parent_vertex_2d_entity, edge_angle, position
            );
            let parent_vertex_3d_entity = vertex_manager
                .vertex_entity_2d_to_3d(parent_vertex_2d_entity)
                .unwrap();
            Some(parent_vertex_3d_entity)
        }
        VertexTypeData::Mesh(_, _) => {
            info!("CreateVertexMesh(position: {:?})", position);
            None
        }
    };
    let (new_vertex_2d_entity, new_vertex_3d_entity) = vertex_manager.create_networked_vertex(
        &mut commands,
        &mut client,
        &mut camera_manager,
        &mut meshes,
        &mut materials,
        file_type_value,
        tab_file_entity,
        parent_vertex_3d_entity_opt,
        position,
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
        Client<Main>,
        ResMut<Canvas>,
        ResMut<CameraManager>,
        ResMut<VertexManager>,
        ResMut<EdgeManager>,
        ResMut<FaceManager>,
        ResMut<Storage<CpuMesh>>,
        ResMut<Storage<CpuMaterial>>,
        Query<&Transform>,
    )> = SystemState::new(world);
    let (
        mut commands,
        mut client,
        mut canvas,
        mut camera_manager,
        mut vertex_manager,
        mut edge_manager,
        mut face_manager,
        mut meshes,
        mut materials,
        transform_q,
    ) = system_state.get_mut(world);

    match vertex_type_data {
        VertexTypeData::Skel(parent_vertex_2d_entity, edge_angle, children_opt) => {
            if let Some(children) = children_opt {
                vertex_manager.create_networked_children_tree(
                    action_stack,
                    &mut commands,
                    &mut client,
                    &mut camera_manager,
                    &mut edge_manager,
                    &mut face_manager,
                    &mut meshes,
                    &mut materials,
                    new_vertex_2d_entity,
                    new_vertex_3d_entity,
                    children,
                    tab_file_entity,
                    &mut entities_to_release,
                );
            }
            let parent_vertex_3d_entity = vertex_manager
                .vertex_entity_2d_to_3d(&parent_vertex_2d_entity)
                .unwrap();
            edge_manager.create_networked_edge(
                &mut commands,
                &mut client,
                &mut camera_manager,
                &mut vertex_manager,
                &mut face_manager,
                &mut meshes,
                &mut materials,
                parent_vertex_2d_entity,
                parent_vertex_3d_entity,
                new_vertex_2d_entity,
                new_vertex_3d_entity,
                tab_file_entity,
                FileExtension::Skel,
                Some(edge_angle),
                &mut entities_to_release,
            );
        }
        VertexTypeData::Mesh(connected_vertex_entities, connected_face_vertex_pairs) => {
            let mut edge_3d_entities = Vec::new();
            for (connected_vertex_2d_entity, old_edge_opt) in connected_vertex_entities {
                let connected_vertex_3d_entity = vertex_manager
                    .vertex_entity_2d_to_3d(&connected_vertex_2d_entity)
                    .unwrap();
                let (new_edge_2d_entity, new_edge_3d_entity) = edge_manager.create_networked_edge(
                    &mut commands,
                    &mut client,
                    &mut camera_manager,
                    &mut vertex_manager,
                    &mut face_manager,
                    &mut meshes,
                    &mut materials,
                    connected_vertex_2d_entity,
                    connected_vertex_3d_entity,
                    new_vertex_2d_entity,
                    new_vertex_3d_entity,
                    tab_file_entity,
                    FileExtension::Mesh,
                    None,
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
                let connected_face_vertex_a_3d = vertex_manager
                    .vertex_entity_2d_to_3d(&connected_face_vertex_a_2d)
                    .unwrap();
                let connected_face_vertex_b_3d = vertex_manager
                    .vertex_entity_2d_to_3d(&connected_face_vertex_b_2d)
                    .unwrap();
                let face_key = FaceKey::new(
                    new_vertex_3d_entity,
                    connected_face_vertex_a_3d,
                    connected_face_vertex_b_3d,
                );

                face_manager.remove_new_face_key(&face_key);
                let new_face_2d_entity = face_manager.process_new_face(
                    &mut commands,
                    &mut camera_manager,
                    &mut vertex_manager,
                    &mut edge_manager,
                    &mut meshes,
                    &mut materials,
                    tab_file_entity,
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
                        tab_file_entity,
                    );
                }
            }
        }
    };

    // select vertex
    input_manager.select_shape(&mut canvas, &new_vertex_2d_entity, CanvasShape::Vertex);
    selected_vertex_3d = new_vertex_3d_entity;
    selected_vertex_2d = new_vertex_2d_entity;

    system_state.apply(world);

    // release all non-selected vertices
    {
        let mut system_state: SystemState<(Commands, Client<Main>)> = SystemState::new(world);
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
