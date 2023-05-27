use std::collections::HashMap;

use bevy_ecs::{
    entity::Entity,
    event::EventReader,
    system::{Commands, Query, Res},
};
use bevy_log::info;

use naia_bevy_client::{
    events::{
        DespawnEntityEvent, InsertComponentEvents, RemoveComponentEvents, SpawnEntityEvent,
        UpdateComponentEvents, EntityAuthGrantedEvent, EntityAuthDeniedEvent, EntityAuthResetEvent,
    },
    Client,
};

use vortex_proto::components::{EntryKind, FileSystemChild, FileSystemEntry, FileSystemRootChild};

use crate::app::{
    components::file_system::{FileSystemParent, FileSystemUiState},
    resources::global::Global,
};

pub fn auth_granted_events(mut event_reader: EventReader<EntityAuthGrantedEvent>) {
    for EntityAuthGrantedEvent(_entity) in event_reader.iter() {
        info!("auth granted for entity");
    }
}

pub fn auth_denied_events(mut event_reader: EventReader<EntityAuthDeniedEvent>) {
    for EntityAuthDeniedEvent(_entity) in event_reader.iter() {
        info!("auth denied for entity");
    }
}

pub fn auth_reset_events(mut event_reader: EventReader<EntityAuthResetEvent>) {
    for EntityAuthResetEvent(_entity) in event_reader.iter() {
        info!("auth reset for entity");
    }
}