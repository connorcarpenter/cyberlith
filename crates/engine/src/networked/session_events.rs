use bevy_ecs::{prelude::World as BevyWorld, event::EventReader};

use naia_bevy_client::events::{DespawnEntityEvent, SpawnEntityEvent};

use logging::info;

use session_server_naia_proto::components::{GlobalChatMessage};

use crate::networked::{insert_component_event::{InsertComponentEvent, insert_component_event, insert_component_events}, client_markers::Session};

pub type SessionInsertComponentEvent<C> = InsertComponentEvent<Session, C>;

// used as a system
pub fn spawn_entity_events(mut event_reader: EventReader<SpawnEntityEvent<Session>>) {
    for _event in event_reader.read() {
        // info!("spawned entity");
    }
}

// used as a system
pub fn despawn_entity_events(mut event_reader: EventReader<DespawnEntityEvent<Session>>) {
    for _event in event_reader.read() {
        // info!("despawned entity");
    }
}

// used as a system
pub fn session_insert_component_events(world: &mut BevyWorld) {
    let events_collection = insert_component_events::<Session>(world);

    for events in events_collection {

        // info!("received session events: [");

        // other events
        insert_component_event::<Session, GlobalChatMessage>(world, &events);

        // info!("]");
    }
}