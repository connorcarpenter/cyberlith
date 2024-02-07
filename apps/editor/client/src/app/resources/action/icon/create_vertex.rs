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
use storage::Assets;

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
    let IconAction::CreateVertex(icon_vertex_data, position, old_vertex_entities_opt) = action else {
        panic!("Expected CreateVertex");
    };
    let frame_entity = icon_vertex_data.frame_entity;

    info!("CreateVertex");

    let mut entities_to_release = Vec::new();
    let deselected_vertex_entity_store;
    let selected_vertex;

    let mut system_state: SystemState<(
        Commands,
        Client<Main>,
        ResMut<Assets<CpuMesh>>,
        ResMut<Assets<CpuMaterial>>,
    )> = SystemState::new(world);
    let (mut commands, mut client, mut meshes, mut materials) = system_state.get_mut(world);

    // deselect all selected vertices
    let deselected_vertex_entity = deselect_selected_shape(icon_manager);
    deselected_vertex_entity_store = deselected_vertex_entity;
    if let Some((entity, _)) = deselected_vertex_entity {
        let mut entity_mut = commands.entity(entity);
        if entity_mut.authority(&client).is_some() {
            entity_mut.release_authority(&mut client);
        }
    }

    // create vertex
    let new_vertex_entity = icon_manager.create_networked_vertex(
        &mut commands,
        &mut client,
        &mut meshes,
        &mut materials,
        &current_file_entity,
        &frame_entity,
        position,
        &mut entities_to_release,
    );

    // migrate undo entities
    if let Some(old_vertex_entity) = old_vertex_entities_opt {
        action_stack.migrate_vertex_entities(old_vertex_entity, new_vertex_entity);
    }

    system_state.apply(world);

    let mut system_state: SystemState<(
        Commands,
        Client<Main>,
        ResMut<Assets<CpuMesh>>,
        ResMut<Assets<CpuMaterial>>,
        Query<&Transform>,
    )> = SystemState::new(world);
    let (mut commands, mut client, mut meshes, mut materials, transform_q) =
        system_state.get_mut(world);

    let mut edge_entities = Vec::new();
    for (connected_vertex_entity, old_edge_opt) in icon_vertex_data.connected_vertices {
        let new_edge_entity = icon_manager.create_networked_edge(
            &mut commands,
            &mut client,
            &mut meshes,
            &mut materials,
            &connected_vertex_entity,
            &new_vertex_entity,
            &current_file_entity,
            &frame_entity,
            &mut entities_to_release,
        );
        edge_entities.push(new_edge_entity);
        if let Some(old_edge_entity) = old_edge_opt {
            action_stack.migrate_edge_entities(old_edge_entity, new_edge_entity);
        }
    }
    for (
        connected_face_vertex_a,
        connected_face_vertex_b,
        old_local_face_entity,
        create_net_face,
    ) in icon_vertex_data.face_data
    {
        let face_key = IconFaceKey::new(
            new_vertex_entity,
            connected_face_vertex_a,
            connected_face_vertex_b,
        );

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

    // select vertex
    icon_manager.select_shape(&new_vertex_entity, CanvasShape::Vertex);
    selected_vertex = new_vertex_entity;

    system_state.apply(world);

    // release all non-selected vertices
    {
        let mut system_state: SystemState<(Commands, Client<Main>)> = SystemState::new(world);
        let (mut commands, mut client) = system_state.get_mut(world);

        for entity_to_release in entities_to_release {
            if entity_to_release != selected_vertex {
                commands
                    .entity(entity_to_release)
                    .release_authority(&mut client);
            }
        }

        system_state.apply(world);
    }

    return vec![IconAction::DeleteVertex(
        selected_vertex,
        deselected_vertex_entity_store,
    )];
}
