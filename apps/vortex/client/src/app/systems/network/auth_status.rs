use bevy_ecs::{
    entity::Entity,
    event::EventReader,
    system::{Query, Res, ResMut},
};
use bevy_log::{info, warn};

use naia_bevy_client::{
    events::{EntityAuthDeniedEvent, EntityAuthGrantedEvent, EntityAuthResetEvent},
    Client,
};

use vortex_proto::components::AnimFrame;

use crate::app::{
    components::OwnedByFileLocal,
    resources::{
        action::FileActions, animation_manager::AnimationManager, edge_manager::EdgeManager,
        face_manager::FaceManager, file_manager::FileManager, shape_manager::ShapeManager,
        tab_manager::TabManager, vertex_manager::VertexManager,
    },
};

pub fn auth_granted_events(
    client: Client,
    file_manager: Res<FileManager>,
    mut file_actions: ResMut<FileActions>,
    vertex_manager: Res<VertexManager>,
    edge_manager: Res<EdgeManager>,
    face_manager: Res<FaceManager>,
    animation_manager: Res<AnimationManager>,
    mut tab_manager: ResMut<TabManager>,
    mut event_reader: EventReader<EntityAuthGrantedEvent>,
    owned_by_q: Query<&OwnedByFileLocal>,
    frame_q: Query<&AnimFrame>,
) {
    for EntityAuthGrantedEvent(entity) in event_reader.iter() {
        process_entity_auth_status(
            &client,
            &file_manager,
            &mut file_actions,
            &vertex_manager,
            &edge_manager,
            &face_manager,
            &animation_manager,
            &mut tab_manager,
            &owned_by_q,
            &frame_q,
            entity,
            "granted",
        );
    }
}

pub fn auth_denied_events(
    client: Client,
    file_manager: Res<FileManager>,
    mut file_actions: ResMut<FileActions>,
    vertex_manager: Res<VertexManager>,
    edge_manager: Res<EdgeManager>,
    face_manager: Res<FaceManager>,
    animation_manager: Res<AnimationManager>,
    mut tab_manager: ResMut<TabManager>,
    mut event_reader: EventReader<EntityAuthDeniedEvent>,
    owned_by_q: Query<&OwnedByFileLocal>,
    frame_q: Query<&AnimFrame>,
) {
    for EntityAuthDeniedEvent(entity) in event_reader.iter() {
        process_entity_auth_status(
            &client,
            &file_manager,
            &mut file_actions,
            &vertex_manager,
            &edge_manager,
            &face_manager,
            &animation_manager,
            &mut tab_manager,
            &owned_by_q,
            &frame_q,
            entity,
            "denied",
        );
    }
}

pub fn auth_reset_events(
    client: Client,
    file_manager: Res<FileManager>,
    mut file_actions: ResMut<FileActions>,
    vertex_manager: Res<VertexManager>,
    edge_manager: Res<EdgeManager>,
    face_manager: Res<FaceManager>,
    animation_manager: Res<AnimationManager>,
    mut tab_manager: ResMut<TabManager>,
    mut event_reader: EventReader<EntityAuthResetEvent>,
    owned_by_q: Query<&OwnedByFileLocal>,
    frame_q: Query<&AnimFrame>,
) {
    for EntityAuthResetEvent(entity) in event_reader.iter() {
        process_entity_auth_status(
            &client,
            &file_manager,
            &mut file_actions,
            &vertex_manager,
            &edge_manager,
            &face_manager,
            &animation_manager,
            &mut tab_manager,
            &owned_by_q,
            &frame_q,
            entity,
            "reset",
        );
    }
}

fn process_entity_auth_status(
    client: &Client,
    file_manager: &FileManager,
    file_actions: &mut FileActions,
    vertex_manager: &VertexManager,
    edge_manager: &EdgeManager,
    face_manager: &FaceManager,
    animation_manager: &AnimationManager,
    tab_manager: &mut TabManager,
    owned_by_q: &Query<&OwnedByFileLocal>,
    frame_q: &Query<&AnimFrame>,
    entity: &Entity,
    status: &str,
) {
    if ShapeManager::has_shape_entity_3d(vertex_manager, edge_manager, face_manager, entity) {
        info!(
            "auth processing for shape entity `{:?}`: `{:?}`",
            entity, status
        );
        if let Ok(owning_file_entity) = owned_by_q.get(*entity) {
            if let Some(tab_state) = tab_manager.tab_state_mut(&owning_file_entity.file_entity) {
                let shape_3d_entity = ShapeManager::shape_entity_3d_to_2d(
                    vertex_manager,
                    edge_manager,
                    face_manager,
                    entity,
                )
                .unwrap();
                tab_state
                    .action_stack
                    .entity_update_auth_status(&shape_3d_entity);
            } else {
                warn!(
                    "no tab state found for file entity: {:?}",
                    owning_file_entity.file_entity
                );
            }
        } else {
            warn!("no owning file entity found for shape entity: {:?}", entity);
        }
    } else if file_manager.entity_is_file(entity) {
        // entity is file
        info!(
            "auth processing for file entity `{:?}`: `{:?}`",
            entity, status
        );
        file_actions.entity_update_auth_status(entity);
    } else if animation_manager.entity_is_frame(entity) {
        info!(
            "auth processing for frame entity `{:?}`: `{:?}`",
            entity, status
        );
        let Ok(frame_component) = frame_q.get(*entity) else {
            panic!("component for frame entity `{:?}` not found", entity);
        };
        let owning_file_entity = frame_component.file_entity.get(client).unwrap();
        if let Some(tab_state) = tab_manager.tab_state_mut(&owning_file_entity) {
            tab_state.action_stack.entity_update_auth_status(&entity);
        } else {
            warn!(
                "no tab state found for file entity: {:?}",
                owning_file_entity
            );
        }
    } else if animation_manager.entity_is_rotation(entity) {
        info!(
            "auth processing for rotation entity `{:?}`: `{:?}`",
            entity, status
        );
        let frame_entity = animation_manager
            .get_rotations_frame_entity(entity)
            .unwrap();
        let Ok(frame_component) = frame_q.get(frame_entity) else {
            panic!("component for rotation entity `{:?}` not found", frame_entity);
        };
        let owning_file_entity = frame_component.file_entity.get(client).unwrap();
        if let Some(tab_state) = tab_manager.tab_state_mut(&owning_file_entity) {
            tab_state.action_stack.entity_update_auth_status(&entity);
        } else {
            warn!(
                "no tab state found for file entity: {:?}",
                owning_file_entity
            );
        }
    } else {
        panic!("unhandled auth status: entity `{:?}`: {:?}", entity, status);
    }
}
