use bevy_ecs::{change_detection::ResMut, event::EventReader, system::Commands, system::Res};

use naia_bevy_server::{
    events::{AuthEvents, ConnectEvent, DisconnectEvent},
    Server,
};

use bevy_http_client::HttpClient;
use logging::{info, warn};

use session_server_naia_proto::messages::Auth;

use crate::{
    asset::{asset_manager::AssetManager, user_load_default_assets},
    session_instance::SessionInstance,
    social::SocialManager,
    user::UserManager,
};

pub fn auth_events(
    mut commands: Commands,
    mut user_manager: ResMut<UserManager>,
    social_manager: Res<SocialManager>,
    mut http_client: ResMut<HttpClient>,
    mut server: Server,
    mut event_reader: EventReader<AuthEvents>,
) {
    for events in event_reader.read() {
        for (user_key, auth) in events.read::<Auth>() {
            if let Some(user_id) = user_manager.take_login_token(&auth.token()) {
                info!("Accepted connection. Token: {}", auth.token());

                let user_presence_room_key = social_manager.user_presence_manager.room_key();

                // add to users
                user_manager.add_connected_user(
                    &mut commands,
                    &mut server,
                    &mut http_client,
                    &user_presence_room_key,
                    user_key,
                    user_id,
                );

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

        // add to user presence room
        let user_presence_room_key = social_manager.user_presence_manager.room_key();
        server.room_mut(&user_presence_room_key).add_user(user_key);

        // add to global chat room
        let global_chat_room_key = social_manager.chat_message_manager.room_key();
        server.room_mut(&global_chat_room_key).add_user(user_key);

        // add to match lobbies room
        let match_lobbies_room_key = social_manager.lobby_manager.room_key();
        server.room_mut(&match_lobbies_room_key).add_user(user_key);

        // Assets

        asset_manager.register_user(user_key);

        // load "default" assets
        user_load_default_assets(&mut server, &mut http_client, &mut asset_manager, user_key);
    }
}

pub fn disconnect_events(
    mut http_client: ResMut<HttpClient>,
    mut user_manager: ResMut<UserManager>,
    mut social_manager: ResMut<SocialManager>,
    mut asset_manager: ResMut<AssetManager>,
    session_instance: Res<SessionInstance>,
    mut event_reader: EventReader<DisconnectEvent>,
) {
    for DisconnectEvent(user_key, user) in event_reader.read() {
        info!("Server disconnected from: {:?}", user.address());

        // TODO: probably need to deregister user from global too?

        // remove from user manager
        let user_id = user_manager.remove_connected_user(user_key).unwrap();

        // remove from asset manager
        asset_manager.deregister_user(user_key);

        let social_server_url = social_manager.get_social_server_url();

        // send user disconnect to social server
        social_manager
            .user_presence_manager
            .send_user_disconnect_req(
                &mut http_client,
                social_server_url.as_ref(),
                &session_instance,
                &user_id,
            );
    }
}
