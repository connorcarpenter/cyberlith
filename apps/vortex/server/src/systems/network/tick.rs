use bevy_ecs::event::EventReader;
use naia_bevy_server::{events::TickEvent, Server};

pub fn tick_events(
    mut server: Server,
    mut tick_reader: EventReader<TickEvent>,
) {
    let mut has_ticked = false;

    for TickEvent(_server_tick) in tick_reader.iter() {
        has_ticked = true;
    }

    if has_ticked {
        // Update scopes of entities
        for (_, user_key, entity) in server.scope_checks() {
            // This indicates the Entity should be in this scope.
            server.user_scope(&user_key).include(&entity);
        }
    }
}