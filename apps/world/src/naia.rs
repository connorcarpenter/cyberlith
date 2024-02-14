use std::{time::Duration, net::SocketAddr};

use bevy_ecs::{system::{Commands, Res}, event::EventReader, change_detection::ResMut};
use bevy_log::{info, warn};

use naia_bevy_server::{events::{AuthEvents, ConnectEvent, DisconnectEvent, ErrorEvent}, transport::webrtc, Server, CommandsExt};

use config::{WORLD_SERVER_SIGNAL_PORT, WORLD_SERVER_WEBRTC_PORT, PUBLIC_IP_ADDR, SELF_BINDING_ADDR};

use world_server_naia_proto::{messages::Auth, components::Body};

use crate::{global::Global, asset_manager::{AssetCatalog, AssetCommandsExt, AssetManager}};

pub fn init(mut commands: Commands, mut server: Server) {
    info!("World Naia Server starting up");

    // set up server
    let server_addresses = webrtc::ServerAddrs::new(
        // IP Address to listen on for WebRTC signaling
        SocketAddr::new(SELF_BINDING_ADDR.parse().unwrap(), WORLD_SERVER_SIGNAL_PORT),
        // IP Address to listen on for UDP WebRTC data channels
        SocketAddr::new(SELF_BINDING_ADDR.parse().unwrap(), WORLD_SERVER_WEBRTC_PORT),
        // The public WebRTC IP address to advertise
        format!("http://{}:{}", PUBLIC_IP_ADDR, WORLD_SERVER_WEBRTC_PORT).as_str(),
    );
    let socket = webrtc::Socket::new(&server_addresses, server.socket_config());
    server.listen(socket);

    // set up global
    let main_room_key = server.make_room().key();
    let registration_resend_rate = Duration::from_secs(5);
    let region_server_disconnect_timeout = Duration::from_secs(16);
    commands.insert_resource(Global::new(main_room_key, registration_resend_rate, region_server_disconnect_timeout));
}

pub fn auth_events(
    mut global: ResMut<Global>,
    mut server: Server,
    mut event_reader: EventReader<AuthEvents>
) {
    for events in event_reader.read() {
        for (user_key, auth) in events.read::<Auth>() {
            if global.take_login_token(&auth.token) {

                info!("Accepted connection. Token: {}", auth.token);

                // Accept incoming connection
                server.accept_connection(&user_key);
            } else {
                // Reject incoming connection
                server.reject_connection(&user_key);

                warn!("Rejected connection. Token: {}", auth.token);
            }
        }
    }
}

pub fn connect_events(
    mut commands: Commands,
    mut server: Server,
    global: Res<Global>,
    mut asset_manager: ResMut<AssetManager>,
    mut event_reader: EventReader<ConnectEvent>
) {
    for ConnectEvent(user_key) in event_reader.read() {
        let address = server.user(user_key).address();

        info!("Server connected to: {}", address);

        let entity = commands
            // Spawn new Entity
            .spawn_empty()
            // MUST call this to begin replication
            .enable_replication(&mut server)
            // insert asset ref
            .insert_asset::<Body>(&mut asset_manager, &mut server, AssetCatalog::Cube.into())
            // return Entity id
            .id();

        server.room_mut(&global.main_room_key()).add_entity(&entity);
    }
}

pub fn disconnect_events(mut event_reader: EventReader<DisconnectEvent>) {
    for DisconnectEvent(_user_key, user) in event_reader.read() {
        info!("Server disconnected from: {:?}", user.address);
    }
}

pub fn error_events(mut event_reader: EventReader<ErrorEvent>) {
    for ErrorEvent(error) in event_reader.read() {
        info!("Server Error: {:?}", error);
    }
}
