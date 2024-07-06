use bevy_ecs::{event::EventReader, prelude::World as BevyWorld};

use naia_bevy_client::events::{DespawnEntityEvent, SpawnEntityEvent};

use session_server_naia_proto::components::{GlobalChatMessage, PublicUserInfo};

use crate::networked::{
    client_markers::Session,
    remove_component_event::{remove_component_event, remove_component_events, RemoveComponentEvent},
    insert_component_event::{
        insert_component_event, insert_component_events, InsertComponentEvent,
    },
    update_component_event::{
        update_component_event, update_component_events, UpdateComponentEvent,
    },
};

pub type SessionInsertComponentEvent<C> = InsertComponentEvent<Session, C>;
pub type SessionUpdateComponentEvent<C> = UpdateComponentEvent<Session, C>;
pub type SessionRemoveComponentEvent<C> = RemoveComponentEvent<Session, C>;

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
        insert_component_event::<Session, PublicUserInfo>(world, &events);

        // info!("]");
    }
}

// used as a system
pub fn session_update_component_events(world: &mut BevyWorld) {
    let events_collection = update_component_events::<Session>(world);

    for events in events_collection {

        // other events
        update_component_event::<Session, GlobalChatMessage>(world, &events);
        update_component_event::<Session, PublicUserInfo>(world, &events);
    }
}

// used as a system
pub fn session_remove_component_events(world: &mut BevyWorld) {
    let events_collection = remove_component_events::<Session>(world);

    for events in events_collection {
        // info!("received session events: [");

        // other events
        remove_component_event::<Session, GlobalChatMessage>(world, &events);
        remove_component_event::<Session, PublicUserInfo>(world, &events);

        // info!("]");
    }
}