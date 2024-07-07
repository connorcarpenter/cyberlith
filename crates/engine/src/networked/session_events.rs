use bevy_app::{App, Plugin, Startup, Update};
use bevy_ecs::{event::EventReader, prelude::World as BevyWorld};

use naia_bevy_client::events::{DespawnEntityEvent, SpawnEntityEvent};

use session_server_naia_proto::components::{GlobalChatMessage, PublicUserInfo};

use crate::{
    networked::{
        client_markers::Session,
        component_events::{
            component_events_startup, get_component_events, InsertComponentEvent, RemoveComponentEvent, UpdateComponentEvent,
        }
    },
    session::{
        SessionDespawnEntityEvent, SessionSpawnEntityEvent
    }
};

pub type SessionInsertComponentEvent<C> = InsertComponentEvent<Session, C>;
pub type SessionUpdateComponentEvent<C> = UpdateComponentEvent<Session, C>;
pub type SessionRemoveComponentEvent<C> = RemoveComponentEvent<Session, C>;

pub struct SessionEventsPlugin;

impl Plugin for SessionEventsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, spawn_entity_events)
            .add_event::<SessionSpawnEntityEvent>()

            .add_systems(Update, despawn_entity_events)
            .add_event::<SessionDespawnEntityEvent>()

            .add_systems(Startup, component_events_startup::<Session>)
            .add_systems(Update, component_events_update)

            .add_event::<SessionInsertComponentEvent<GlobalChatMessage>>()
            .add_event::<SessionUpdateComponentEvent<GlobalChatMessage>>()
            .add_event::<SessionRemoveComponentEvent<GlobalChatMessage>>()

            .add_event::<SessionInsertComponentEvent<PublicUserInfo>>()
            .add_event::<SessionUpdateComponentEvent<PublicUserInfo>>()
            .add_event::<SessionRemoveComponentEvent<PublicUserInfo>>();
    }
}

// used as a system
fn spawn_entity_events(mut event_reader: EventReader<SpawnEntityEvent<Session>>) {
    for _event in event_reader.read() {
        // info!("spawned entity");
    }
}

// used as a system
fn despawn_entity_events(mut event_reader: EventReader<DespawnEntityEvent<Session>>) {
    for _event in event_reader.read() {
        // info!("despawned entity");
    }
}

// used as a system
pub fn component_events_update(world: &mut BevyWorld) {

    for events in get_component_events::<Session>(world) {
        events.process::<GlobalChatMessage>(world);
        events.process::<PublicUserInfo>(world);
    }
}