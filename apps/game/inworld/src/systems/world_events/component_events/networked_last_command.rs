use bevy_ecs::{
    event::EventReader,
    system::{Query, ResMut},
};

use game_engine::logging::info;

use game_app_network::world::{
    components::NetworkedLastCommand, WorldInsertComponentEvent, WorldRemoveComponentEvent,
    WorldUpdateComponentEvent,
};

use crate::resources::RollbackManager;

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
    mut rollback_manager: ResMut<RollbackManager>,
    last_command_q: Query<&NetworkedLastCommand>,
    mut event_reader: EventReader<WorldUpdateComponentEvent<NetworkedLastCommand>>,
) {
    for event in event_reader.read() {
        let entity = event.entity;
        let server_tick = event.tick;

        let last_command = last_command_q.get(entity).unwrap();

        info!(
            "received Updated Component: `NetworkedLastCommand` from World Server! (entity: {:?}, move_dir: {:?})",
            entity,
            last_command.get(),
        );

        rollback_manager.add_event(server_tick);
    }
}

pub fn remove_net_last_command_events(
    mut event_reader: EventReader<WorldRemoveComponentEvent<NetworkedLastCommand>>,
) {
    for _event in event_reader.read() {
        info!("removed NetworkedLastCommand component from entity");
    }
}
