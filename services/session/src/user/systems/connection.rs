use bevy_ecs::{change_detection::ResMut, event::EventReader, system::Commands, system::Res};

use naia_bevy_server::{
    events::{AuthEvents, ConnectEvent, DisconnectEvent},
    Server,
};

use bevy_http_client::HttpClient;
use logging::{info, warn};

use session_server_naia_proto::messages::Auth;

use crate::{
    asset::{asset_manager::AssetManager},
    session_instance::SessionInstance,
    social::SocialManager,
    user::UserManager,
};

pub fn auth_events(
    mut user_manager: ResMut<UserManager>,
    mut server: Server,
    mut event_reader: EventReader<AuthEvents>,
) {
    for events in event_reader.read() {
        for (user_key, auth) in events.read::<Auth>() {
            if let Some(user_id) = user_manager.spend_login_token(&auth.token()) {
                info!("Accepted connection. Token: {}", auth.token());

                user_manager.accept_user(user_key, user_id);

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
    mut commands: Commands,
    mut server: Server,
    mut http_client: ResMut<HttpClient>,
    mut user_manager: ResMut<UserManager>,
    social_manager: Res<SocialManager>,
    mut asset_manager: ResMut<AssetManager>,

    mut event_reader: EventReader<ConnectEvent>,
) {
    for ConnectEvent(user_key) in event_reader.read() {
        let address = server.user(user_key).address();

        info!("Server connected to User: {}", address);

        let main_menu_room_key = social_manager.global_room_key().unwrap();

        user_manager.connect_user(
            &mut commands,
            &mut server,
            &mut http_client,
            &user_key,
            &main_menu_room_key,
        );

        // add to main menu room
        server.room_mut(&main_menu_room_key).add_user(user_key);

        // Assets
        asset_manager.register_user(user_key);

        cfg_if::cfg_if!(
            if #[cfg(feature = "odst")] {} else {
                // load "default" assets
                asset::user_load_default_assets(&mut server, &mut http_client, &mut asset_manager, user_key);
            }
        );
    }
}

pub fn disconnect_events(
    mut commands: Commands,
    mut naia_server: Server,
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
        let user_id = user_manager
            .disconnect_user(&mut commands, &mut naia_server, user_key)
            .unwrap();

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
