use std::net::SocketAddr;

use bevy_ecs::{change_detection::ResMut, event::EventReader};
use bevy_ecs::system::Res;

use naia_bevy_server::{
    events::{AuthEvents, ConnectEvent, DisconnectEvent, ErrorEvent},
    transport::webrtc,
    Server,
};
use naia_bevy_server::events::MessageEvents;
use asset_id::AssetId;
use bevy_http_client::HttpClient;
use bevy_http_server::HttpServer;

use config::{
    PUBLIC_IP_ADDR, PUBLIC_PROTOCOL, SELF_BINDING_ADDR, SESSION_SERVER_SIGNAL_PORT,
    SESSION_SERVER_WEBRTC_PORT,
};
use logging::{info, warn};

use session_server_naia_proto::channels::ClientActionsChannel;
use session_server_naia_proto::messages::{Auth, WorldConnectRequest};

use crate::{asset::asset_manager::AssetManager, global::Global};
use crate::asset::AssetCatalog;

pub fn init(mut server: Server) {
    info!("Session Naia Server starting up");

    let server_addresses = webrtc::ServerAddrs::new(
        // IP Address to listen on for WebRTC signaling
        SocketAddr::new(
            SELF_BINDING_ADDR.parse().unwrap(),
            SESSION_SERVER_SIGNAL_PORT,
        ),
        // IP Address to listen on for UDP WebRTC data channels
        SocketAddr::new(
            SELF_BINDING_ADDR.parse().unwrap(),
            SESSION_SERVER_WEBRTC_PORT,
        ),
        // The public WebRTC IP address to advertise
        format!(
            "{}://{}:{}",
            PUBLIC_PROTOCOL, PUBLIC_IP_ADDR, SESSION_SERVER_WEBRTC_PORT
        ).as_str(),
    );
    let socket = webrtc::Socket::new(&server_addresses, server.socket_config());
    server.listen(socket);
}

pub fn auth_events(
    mut global: ResMut<Global>,
    mut server: Server,
    mut event_reader: EventReader<AuthEvents>,
) {
    for events in event_reader.read() {
        for (user_key, auth) in events.read::<Auth>() {
            if let Some(user_data) = global.take_login_token(&auth.token()) {
                info!("Accepted connection. Token: {}", auth.token());

                // add to users
                global.add_user(user_key, user_data);

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
    mut event_reader: EventReader<ConnectEvent>,
    mut asset_manager: ResMut<AssetManager>,
    global: Res<Global>,
    mut http_client: ResMut<HttpClient>,
) {
    for ConnectEvent(user_key) in event_reader.read() {
        let address = server.user(user_key).address();

        info!("Server connected to: {}", address);

        asset_manager.register_user(user_key);

        asset_manager.load_user_asset(&mut server, &mut http_client, global.get_asset_server_url(), *user_key, &AssetCatalog::game_main_menu_ui());
        asset_manager.load_user_asset(&mut server, &mut http_client, global.get_asset_server_url(), *user_key, &AssetCatalog::game_host_match_ui());
    }
}

pub fn disconnect_events(
    mut event_reader: EventReader<DisconnectEvent>,
    mut asset_manager: ResMut<AssetManager>,
) {
    for DisconnectEvent(user_key, user) in event_reader.read() {
        info!("Server disconnected from: {:?}", user.address());

        // TODO: probably need to deregister user from global?

        asset_manager.deregister_user(user_key);
    }
}

pub fn error_events(mut event_reader: EventReader<ErrorEvent>) {
    for ErrorEvent(error) in event_reader.read() {
        info!("Server Error: {:?}", error);
    }
}

pub fn message_events(
    mut global: ResMut<Global>,
    mut event_reader: EventReader<MessageEvents>,
) {
    for events in event_reader.read() {
        for (user_key, _req) in events.read::<ClientActionsChannel, WorldConnectRequest>() {
            if let Some(user_data) = global.get_user_data_mut(&user_key) {
                user_data.make_ready_for_world_connect();
            } else {
                warn!("User not found: {:?}", user_key);
            }
        }
    }
}
