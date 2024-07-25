use bevy_ecs::{prelude::Query, event::EventReader, change_detection::ResMut};

use game_engine::world::{behavior as shared_behavior, messages::KeyCommand, components::Position, WorldClient, WorldClientTickEvent, channels::PlayerCommandChannel};

use crate::resources::Global;

pub fn tick_events(
    mut client: WorldClient,
    mut global: ResMut<Global>,
    mut tick_reader: EventReader<WorldClientTickEvent>,
    mut position_q: Query<&mut Position>,
) {
    let Some(predicted_entity) = global
        .owned_entity
        .as_ref()
        .map(|owned_entity| owned_entity.predicted)
    else {
        // No owned Entity
        return;
    };

    let Some(command) = global.queued_command.take() else {
        return;
    };

    for event in tick_reader.read() {
        let client_tick = event.tick;

        // Command History
        if !global.command_history.can_insert(&client_tick) {
            // History is full
            continue;
        }

        // Record command
        global.command_history.insert(client_tick, command.clone());

        // Send command
        client.send_tick_buffer_message::<PlayerCommandChannel, KeyCommand>(&client_tick, &command);

        if let Ok(mut position) = position_q.get_mut(predicted_entity) {
            // Apply command
            shared_behavior::process_command(&command, &mut position);
        }
    }
}