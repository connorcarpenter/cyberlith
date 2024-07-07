use bevy_app::{App, Plugin, Startup, Update};
use bevy_ecs::{event::EventReader, prelude::World as BevyWorld};

use naia_bevy_client::events::{DespawnEntityEvent, SpawnEntityEvent};

use session_server_naia_proto::components::{GlobalChatMessage, PublicUserInfo};

use crate::{
    networked::{
        client_markers::Session,
        component_events::{
            insert_component_event, component_events_startup, insert_component_events, InsertComponentEvent, remove_component_event, remove_component_events, RemoveComponentEvent, update_component_event, update_component_events, UpdateComponentEvent,
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
            .add_event::<SessionInsertComponentEvent<PublicUserInfo>>();
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

    // insert

    let events_collection = insert_component_events::<Session>(world);

    for events in events_collection {

        insert_component_event::<Session, GlobalChatMessage>(world, &events);
        insert_component_event::<Session, PublicUserInfo>(world, &events);
    }

    // update

    let events_collection = update_component_events::<Session>(world);

    for events in events_collection {

        update_component_event::<Session, GlobalChatMessage>(world, &events);
        update_component_event::<Session, PublicUserInfo>(world, &events);
    }

    // remove

    let events_collection = remove_component_events::<Session>(world);

    for events in events_collection {

        remove_component_event::<Session, GlobalChatMessage>(world, &events);
        remove_component_event::<Session, PublicUserInfo>(world, &events);
    }
}