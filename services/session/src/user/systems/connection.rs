use bevy_ecs::{change_detection::ResMut, event::EventReader, system::Res};
use bevy_ecs::system::Commands;

use naia_bevy_server::{
    events::{AuthEvents, ConnectEvent, DisconnectEvent},
    Server,
};

use bevy_http_client::HttpClient;
use logging::{info, warn};

use session_server_naia_proto::messages::Auth;

use crate::{
    asset::{asset_manager::AssetManager, user_load_default_assets},
    social::SocialManager,
    user::UserManager,
    session_instance::SessionInstance,
};

pub fn auth_events(
    mut commands: Commands,
    mut user_manager: ResMut<UserManager>,
    social_manager: Res<SocialManager>,
    mut server: Server,
    mut event_reader: EventReader<AuthEvents>,
) {
    for events in event_reader.read() {
        for (user_key, auth) in events.read::<Auth>() {
            if let Some(user_id) = user_manager.take_login_token(&auth.token()) {
                info!("Accepted connection. Token: {}", auth.token());

                let global_chat_room_key = social_manager.get_global_chat_room_key();

                // add to users
                user_manager.add_connected_user(
                    &mut commands,
                    &mut server,
                    &global_chat_room_key,
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

        // send user disconnect to social server
        social_manager.send_user_disconnect_req(
            &mut http_client,
            &session_instance,
            &user_id,
        );
    }
}
