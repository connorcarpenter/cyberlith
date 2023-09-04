
use bevy_ecs::{entity::Entity, event::EventReader, system::{Query, ResMut}};
use bevy_log::{info, warn};

use naia_bevy_client::events::{
    EntityAuthDeniedEvent, EntityAuthGrantedEvent, EntityAuthResetEvent,
};

use crate::app::{components::OwnedByFileLocal, resources::{tab_manager::TabManager, action::FileActions, shape_manager::ShapeManager}};

pub fn auth_granted_events(
    mut file_actions: ResMut<FileActions>,
    mut shape_manager: ResMut<ShapeManager>,
    mut tab_manager: ResMut<TabManager>,
    mut event_reader: EventReader<EntityAuthGrantedEvent>,
    owned_by_q: Query<&OwnedByFileLocal>,
) {
    for EntityAuthGrantedEvent(entity) in event_reader.iter() {
        process_entity_auth_status(&mut file_actions, &mut shape_manager, &mut tab_manager, &owned_by_q, entity, "granted");
    }
}

pub fn auth_denied_events(
    mut file_actions: ResMut<FileActions>,
    mut shape_manager: ResMut<ShapeManager>,
    mut tab_manager: ResMut<TabManager>,
    mut event_reader: EventReader<EntityAuthDeniedEvent>,
    owned_by_q: Query<&OwnedByFileLocal>,
) {
    for EntityAuthDeniedEvent(entity) in event_reader.iter() {
        process_entity_auth_status(&mut file_actions, &mut shape_manager, &mut tab_manager, &owned_by_q, entity, "denied");
    }
}

pub fn auth_reset_events(
    mut file_actions: ResMut<FileActions>,
    mut shape_manager: ResMut<ShapeManager>,
    mut tab_manager: ResMut<TabManager>,
    mut event_reader: EventReader<EntityAuthResetEvent>,
    owned_by_q: Query<&OwnedByFileLocal>,
) {
    for EntityAuthResetEvent(entity) in event_reader.iter() {
        process_entity_auth_status(&mut file_actions, &mut shape_manager, &mut tab_manager, &owned_by_q, entity, "reset");
    }
}

fn process_entity_auth_status(
    file_actions: &mut FileActions,
    shape_manager: &mut ShapeManager,
    tab_manager: &mut TabManager,
    owned_by_q: &Query<&OwnedByFileLocal>,
    entity: &Entity,
    status: &str,
) {
    if shape_manager.has_shape_entity_3d(entity) {
        info!("auth processing for shape entity `{:?}`: `{:?}`", entity, status);
        if let Ok(owning_file_entity) = owned_by_q.get(*entity) {
            if let Some(tab_state) = tab_manager.tab_state_mut(&owning_file_entity.file_entity) {
                let shape_3d_entity = shape_manager.shape_entity_3d_to_2d(entity).unwrap();
                tab_state.action_stack.entity_update_auth_status(&shape_3d_entity);
            } else {
                warn!("no tab state found for file entity: {:?}", owning_file_entity.file_entity);
            }
        } else {
            warn!("no owning file entity found for entity: {:?}", entity);
        }
    } else {
        // entity is .. file?
        info!("auth processing for file? entity `{:?}`: `{:?}`", entity, status);
        file_actions.entity_update_auth_status(entity);
    }
}