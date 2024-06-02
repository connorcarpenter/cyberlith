
use bevy_ecs::{system::Res, change_detection::ResMut, event::EventReader};

use naia_bevy_server::{events::{AuthEvents, ConnectEvent, DisconnectEvent}, Server};

use bevy_http_client::HttpClient;
use logging::{info, warn};

use session_server_naia_proto::messages::Auth;

use crate::{social::SocialManager, asset::{user_load_default_assets, asset_manager::AssetManager}, user::UserManager};

pub fn auth_events(
    mut user_manager: ResMut<UserManager>,
    mut server: Server,
    mut event_reader: EventReader<AuthEvents>,
) {
    for events in event_reader.read() {
        for (user_key, auth) in events.read::<Auth>() {
            if let Some(user_data) = user_manager.take_login_token(&auth.token()) {
                info!("Accepted connection. Token: {}", auth.token());

                // add to users
                user_manager.add_user(user_key, user_data);

                // Accept incoming connection
                server.accept_connection(&user_key);
            } else {
                // Reject incoming connection
                server.reject_connection(&user_key);

                warn!("Rejected connection. Token: {}", auth.token());
            }
        }
    }
}

pub fn connect_events(
    mut server: Server,
    mut http_client: ResMut<HttpClient>,
    social_manager: Res<SocialManager>,
    mut asset_manager: ResMut<AssetManager>,

    mut event_reader: EventReader<ConnectEvent>,
) {
    for ConnectEvent(user_key) in event_reader.read() {
        let address = server.user(user_key).address();

        info!("Server connected to: {}", address);

        // add to global chat room
        let global_chat_room_key = social_manager.get_global_chat_room_key();
        server.room_mut(&global_chat_room_key).add_user(user_key);

        // Assets

        asset_manager.register_user(user_key);

        // load "default" assets
        user_load_default_assets(&mut server, &mut http_client, &mut asset_manager, user_key);
    }
}

pub fn disconnect_events(
    mut event_reader: EventReader<DisconnectEvent>,
    mut asset_manager: ResMut<AssetManager>,
) {
    for DisconnectEvent(user_key, user) in event_reader.read() {
        info!("Server disconnected from: {:?}", user.address());

        // TODO: probably need to deregister user from global too?

        asset_manager.deregister_user(user_key);
    }
}