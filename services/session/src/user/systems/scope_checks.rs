
use bevy_ecs::{event::EventReader};

use naia_bevy_server::{events::{TickEvent}, Server};

pub fn scope_checks(
    mut server: Server,
    mut tick_reader: EventReader<TickEvent>,
) {
    let mut has_ticked = false;

    for TickEvent(_server_tick) in tick_reader.read() {
        has_ticked = true;
    }

    if has_ticked {
        // Update scopes of entities
        for (_, user_key, entity) in server.scope_checks() {
            // You'd normally do whatever checks you need to in here..
            // to determine whether each Entity should be in scope or not.

            if !server.user_scope(&user_key).has(&entity) {
                // This indicates the Entity should be in this scope.
                server.user_scope_mut(&user_key).include(&entity);
            }

            // And call this if Entity should NOT be in this scope.
            // server.user_scope_mut(..).exclude(..);
        }
    }
}