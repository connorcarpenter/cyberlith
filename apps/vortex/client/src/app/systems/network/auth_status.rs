use bevy_ecs::{
    event::EventReader,
    system::ResMut,
};
use bevy_log::info;

use naia_bevy_client::{
    events::{EntityAuthDeniedEvent, EntityAuthGrantedEvent, EntityAuthResetEvent},
};

use crate::app::resources::{action_stack::ActionStack, vertex_manager::VertexManager};

pub fn auth_granted_events(
    mut action_stack: ResMut<ActionStack>,
    mut vertex_manager: ResMut<VertexManager>,
    mut event_reader: EventReader<EntityAuthGrantedEvent>,
) {
    for EntityAuthGrantedEvent(entity) in event_reader.iter() {
        if vertex_manager.has_edge_entity_3d(entity) {
            // entity is edge
            // TODO: handle edge auth?
            info!("auth granted for edge entity: {:?}", entity);
        } else if vertex_manager.has_vertex_entity_3d(entity) {
            // entity is vertex
            info!("auth granted for vertex entity: {:?}", entity);
            action_stack.entity_update_auth_status(&mut vertex_manager, entity);
        } else {
            // entity is .. file?
            info!("auth granted for file? entity: {:?}", entity);
            action_stack.entity_update_auth_status(&mut vertex_manager, entity);
        }
    }
}

pub fn auth_denied_events(
    mut action_stack: ResMut<ActionStack>,
    mut vertex_manager: ResMut<VertexManager>,
    mut event_reader: EventReader<EntityAuthDeniedEvent>,
) {
    for EntityAuthDeniedEvent(entity) in event_reader.iter() {
        info!("auth denied for entity: {:?}", entity);

        action_stack.entity_update_auth_status(&mut vertex_manager, entity);
    }
}

pub fn auth_reset_events(
    mut action_stack: ResMut<ActionStack>,
    mut vertex_manager: ResMut<VertexManager>,
    mut event_reader: EventReader<EntityAuthResetEvent>,
) {
    for EntityAuthResetEvent(entity) in event_reader.iter() {
        info!("auth reset for entity: {:?}", entity);

        action_stack.entity_update_auth_status(&mut vertex_manager, entity);
    }
}
