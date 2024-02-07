use bevy_ecs::{
    prelude::{Commands, Entity, Query, World},
    system::{ResMut, SystemState},
};
use bevy_log::info;

use naia_bevy_client::{Client, CommandsExt};

use render_api::{
    base::{CpuMaterial, CpuMesh},
    components::Transform,
};
use storage::Storage;

use crate::app::{
    plugin::Main,
    resources::{
        action::{
            icon::{select_shape::deselect_selected_shape, IconAction},
            ActionStack,
        },
        icon_data::IconFaceKey,
        icon_manager::IconManager,
        shape_data::CanvasShape,
    },
};

pub(crate) fn execute(
    world: &mut World,
    icon_manager: &mut IconManager,
    action_stack: &mut ActionStack<IconAction>,
    current_file_entity: Entity,
    action: IconAction,
) -> Vec<IconAction> {
    let IconAction::CreateEdge(
        frame_entity,
        vertex_entity_a,
        vertex_entity_b,
        shape_to_select,
        face_to_create_opt,
        old_edge_entities_opt,
    ) = action else {
        panic!("Expected CreateEdge");
    };

    let (mut shape_entity_to_select, shape_type_to_select) = shape_to_select;

    info!(
        "CreateEdge(vertex_a: {:?}, vertex_b: {:?}, (shape_entity_to_select: {:?}, {:?}), face_to_create_opt: {:?})",
        vertex_entity_a, vertex_entity_b, shape_entity_to_select, shape_type_to_select, face_to_create_opt
    );

    let mut entities_to_release = Vec::new();
    let selected_shape;
    let deselected_shape_entity_store;
    let created_edge_entity;
    let mut edge_entities = Vec::new();

    {
        let mut system_state: SystemState<(
            Commands,
            Client<Main>,
            ResMut<Storage<CpuMesh>>,
            ResMut<Storage<CpuMaterial>>,
        )> = SystemState::new(world);
        let (mut commands, mut client, mut meshes, mut materials) = system_state.get_mut(world);

        // deselect all selected vertices
        let deselected_shape_entity = deselect_selected_shape(icon_manager);
        deselected_shape_entity_store = deselected_shape_entity;
        if let Some((entity, _)) = deselected_shape_entity {
            let mut entity_mut = commands.entity(entity);
            if entity_mut.authority(&client).is_some() {
                entity_mut.release_authority(&mut client);
            }
        }

        // create edge
        let new_edge_entity = icon_manager.create_networked_edge(
            &mut commands,
            &mut client,
            &mut meshes,
            &mut materials,
            &vertex_entity_a,
            &vertex_entity_b,
            &current_file_entity,
            &frame_entity,
            &mut entities_to_release,
        );
        created_edge_entity = new_edge_entity;
        edge_entities.push(new_edge_entity);

        // migrate undo entities
        if let Some(old_edge_entity) = old_edge_entities_opt {
            action_stack.migrate_edge_entities(old_edge_entity, new_edge_entity);
            if shape_type_to_select == CanvasShape::Edge {
                if shape_entity_to_select == old_edge_entity {
                    shape_entity_to_select = new_edge_entity;
                }
            }
        }

        // select vertex
        icon_manager.select_shape(&shape_entity_to_select, shape_type_to_select);
        selected_shape = shape_entity_to_select;

        system_state.apply(world);
    }

    // release all non-selected shapes
    {
        let mut system_state: SystemState<(Commands, Client<Main>)> = SystemState::new(world);
        let (mut commands, mut client) = system_state.get_mut(world);

        for entity_to_release in entities_to_release {
            if entity_to_release != selected_shape {
                commands
                    .entity(entity_to_release)
                    .release_authority(&mut client);
            }
        }

        system_state.apply(world);
    }

    // create face if necessary
    {
        if let Some(vertex_entities) = face_to_create_opt {
            let mut system_state: SystemState<(
                Commands,
                Client<Main>,
                ResMut<Storage<CpuMesh>>,
                ResMut<Storage<CpuMaterial>>,
                Query<&Transform>,
            )> = SystemState::new(world);
            let (mut commands, mut client, mut meshes, mut materials, transform_q) =
                system_state.get_mut(world);

            for (vertex_of_face_to_create, old_local_face_entity, create_net_face) in
                vertex_entities
            {
                let face_key =
                    IconFaceKey::new(vertex_entity_a, vertex_entity_b, vertex_of_face_to_create);

                icon_manager.remove_new_face_key(&face_key);
                let new_face_entity = icon_manager.process_new_local_face(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    &current_file_entity,
                    &frame_entity,
                    &face_key,
                );
                action_stack.migrate_face_entities(old_local_face_entity, new_face_entity);
                if let Some(palette_color_entity) = create_net_face {
                    icon_manager.create_networked_face(
                        &mut commands,
                        &mut client,
                        &mut meshes,
                        &mut materials,
                        &transform_q,
                        &face_key,
                        [edge_entities[0], edge_entities[1], edge_entities[2]],
                        &current_file_entity,
                        &frame_entity,
                        &palette_color_entity,
                    );
                }
            }

            system_state.apply(world);
        }
    }

    return vec![
        IconAction::DeleteEdge(created_edge_entity, deselected_shape_entity_store),
        IconAction::SelectShape(Some((created_edge_entity, CanvasShape::Edge))),
    ];
}
