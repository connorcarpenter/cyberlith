use bevy_ecs::{event::EventReader};

use game_engine::logging::info;

use game_app_network::world::{
    components::NetworkedLastCommand, WorldInsertComponentEvent, WorldRemoveComponentEvent,
    WorldUpdateComponentEvent,
};

pub fn insert_net_last_command_events(
    mut event_reader: EventReader<WorldInsertComponentEvent<NetworkedLastCommand>>,
) {
    for event in event_reader.read() {
        let entity = event.entity;

        info!(
            "received Inserted Component: `NetworkedLastCommand` from World Server! (entity: {:?})",
            entity
        );
    }
}

pub fn update_net_last_command_events(
    mut event_reader: EventReader<WorldUpdateComponentEvent<NetworkedLastCommand>>,
) {
    for event in event_reader.read() {
        let entity = event.entity;

        info!(
            "received Updated Component: `NetworkedLastCommand` from World Server! (entity: {:?})",
            entity
        );
    }
}

pub fn remove_net_last_command_events(
    mut event_reader: EventReader<WorldRemoveComponentEvent<NetworkedLastCommand>>,
) {
    for _event in event_reader.read() {
        info!("removed NetworkedLastCommand component from entity");
    }
}
