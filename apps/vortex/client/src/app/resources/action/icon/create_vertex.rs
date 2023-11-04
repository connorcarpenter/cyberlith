use bevy_ecs::{
    event::EventWriter,
    prelude::{Commands, Entity, Query, World},
    system::{ResMut, SystemState},
};
use bevy_log::info;

use naia_bevy_client::{Client, CommandsExt};

use render_api::{
    base::{CpuMaterial, CpuMesh},
    components::Transform,
    Assets,
};

use vortex_proto::components::FileExtension;

use crate::app::{
    components::VertexTypeData,
    events::ShapeColorResyncEvent,
    resources::{
        action::{
            icon::{select_shape::deselect_selected_shape, IconAction},
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
    action_stack: &mut ActionStack<IconAction>,
    tab_file_entity: Entity,
    action: IconAction,
) -> Vec<IconAction> {
    let IconAction::CreateVertex(icon_vertex_data, position, old_vertex_entities_opt) = action else {
        panic!("Expected CreateVertex");
    };

    let mut entities_to_release = Vec::new();
    let deselected_vertex_2d_entity_store;
    let selected_vertex_3d;
    let selected_vertex_2d;

    let mut system_state: SystemState<(
        Commands,
        Client,
        ResMut<Canvas>,
        ResMut<CameraManager>,
        ResMut<VertexManager>,
        ResMut<EdgeManager>,
        ResMut<FaceManager>,
        ResMut<Assets<CpuMesh>>,
        ResMut<Assets<CpuMaterial>>,
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

    // create vertex
    let new_vertex_2d_entity = icon_manager.create_networked_vertex(
        &mut commands,
        &mut client,
        &mut camera_manager,
        &mut meshes,
        &mut materials,
        tab_file_entity,
        position,
        &mut entities_to_release,
    );

    // migrate undo entities
    if let Some(old_vertex_2d_entity) = old_vertex_entities_opt {
        action_stack.migrate_vertex_entities(
            old_vertex_2d_entity,
            new_vertex_2d_entity,
        );
    }

    system_state.apply(world);

    let mut system_state: SystemState<(
        Commands,
        Client,
        ResMut<Canvas>,
        ResMut<CameraManager>,
        ResMut<VertexManager>,
        ResMut<EdgeManager>,
        ResMut<FaceManager>,
        ResMut<Assets<CpuMesh>>,
        ResMut<Assets<CpuMaterial>>,
        EventWriter<ShapeColorResyncEvent>,
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
        mut shape_color_resync_events,
        transform_q,
    ) = system_state.get_mut(world);

    let mut edge_3d_entities = Vec::new();
    for (connected_vertex_2d_entity, old_edge_opt) in icon_vertex_data.connected_vertices {
        let new_edge_2d_entity = icon_manager.create_networked_edge(
            &mut commands,
            &mut client,
            &mut camera_manager,
            &mut vertex_manager,
            &mut face_manager,
            &mut meshes,
            &mut materials,
            &mut shape_color_resync_events,
            connected_vertex_2d_entity,
            new_vertex_2d_entity,
            tab_file_entity,
            &mut entities_to_release,
        );
        if let Some(old_edge_2d_entity) = old_edge_opt {
            action_stack.migrate_edge_entities(old_edge_2d_entity, new_edge_2d_entity);
        }
    }
    for (
        connected_face_vertex_a_2d,
        connected_face_vertex_b_2d,
        old_face_2d_entity,
        create_face_3d,
    ) in icon_vertex_data.face_data
    {
        let face_key = FaceKey::new(
            new_vertex_2d_entity,
            connected_face_vertex_a_2d,
            connected_face_vertex_b_2d,
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
                [
                    edge_3d_entities[0],
                    edge_3d_entities[1],
                    edge_3d_entities[2],
                ],
                tab_file_entity,
            );
        }
    }

    // select vertex
    input_manager.select_shape(&mut canvas, &new_vertex_2d_entity, CanvasShape::Vertex);
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

    return vec![IconAction::DeleteVertex(
        selected_vertex_2d,
        deselected_vertex_2d_entity_store,
    )];
}
