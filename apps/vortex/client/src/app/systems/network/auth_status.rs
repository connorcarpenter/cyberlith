use bevy_ecs::{event::EventReader, system::ResMut};
use bevy_log::info;

use naia_bevy_client::events::{
    EntityAuthDeniedEvent, EntityAuthGrantedEvent, EntityAuthResetEvent,
};

use crate::app::resources::{action_stack::ActionStack, vertex_manager::VertexManager};

pub fn auth_granted_events(
    mut action_stack: ResMut<ActionStack>,
    mut vertex_manager: ResMut<VertexManager>,
    mut event_reader: EventReader<EntityAuthGrantedEvent>,
) {
    for EntityAuthGrantedEvent(entity) in event_reader.iter() {
        info!("auth granted for entity");

        action_stack.entity_update_auth_status(&mut vertex_manager, entity);
    }
}

pub fn auth_denied_events(
    mut action_stack: ResMut<ActionStack>,
    mut vertex_manager: ResMut<VertexManager>,
    mut event_reader: EventReader<EntityAuthDeniedEvent>,
) {
    for EntityAuthDeniedEvent(entity) in event_reader.iter() {
        info!("auth denied for entity");

        action_stack.entity_update_auth_status(&mut vertex_manager, entity);
    }
}

pub fn auth_reset_events(
    mut action_stack: ResMut<ActionStack>,
    mut vertex_manager: ResMut<VertexManager>,
    mut event_reader: EventReader<EntityAuthResetEvent>,
) {
    for EntityAuthResetEvent(entity) in event_reader.iter() {
        info!("auth reset for entity");

        action_stack.entity_update_auth_status(&mut vertex_manager, entity);
    }
}
