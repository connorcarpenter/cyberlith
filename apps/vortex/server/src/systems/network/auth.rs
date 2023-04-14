use bevy_ecs::{event::EventReader, system::ResMut};

use naia_bevy_server::{events::AuthEvents, Server};

use vortex_proto::messages::Auth;

use crate::resources::UserManager;

pub fn auth_events(
    mut server: Server,
    mut event_reader: EventReader<AuthEvents>,
    mut user_manager: ResMut<UserManager>,
) {
    for events in event_reader.iter() {
        for (user_key, auth) in events.read::<Auth>() {
            if user_manager.validate_user(&auth.username, &auth.password) {
                // Store user information
                user_manager.add_user(&user_key, &auth.username);

                // Accept incoming connection
                server.accept_connection(&user_key);
            } else {
                // Reject incoming connection
                server.reject_connection(&user_key);
            }
        }
    }
}
