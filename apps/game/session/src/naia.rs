
use bevy_ecs::{event::EventReader, change_detection::ResMut};
use bevy_log::{info, warn};

use naia_bevy_server::{
    events::{AuthEvents, ConnectEvent, DisconnectEvent, ErrorEvent},
    transport::webrtc,
    Server,
};

use bevy_http_client::HttpClient;

use session_server_naia_proto::messages::Auth;
use region_server_http_proto::WorldUserLoginRequest;
use config::{SESSION_SERVER_SIGNAL_ADDR, SESSION_SERVER_WEBRTC_ADDR, REGION_SERVER_ADDR};

use crate::global::Global;

pub fn init(mut server: Server) {
    info!("Session Naia Server starting up");

    let server_addresses = webrtc::ServerAddrs::new(
        SESSION_SERVER_SIGNAL_ADDR
            .parse()
            .expect("could not parse Signaling address/port"),
        // IP Address to listen on for UDP WebRTC data channels
        SESSION_SERVER_WEBRTC_ADDR
            .parse()
            .expect("could not parse WebRTC data address/port"),
        // The public WebRTC IP address to advertise
        format!("http://{}", SESSION_SERVER_WEBRTC_ADDR).as_str(),
    );
    let socket = webrtc::Socket::new(&server_addresses, server.socket_config());
    server.listen(socket);
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
    server: Server,
    mut event_reader: EventReader<ConnectEvent>,
    mut http_client: ResMut<HttpClient>,
    mut global: ResMut<Global>,
) {
    for ConnectEvent(user_key) in event_reader.read() {
        let address = server.user(user_key).address();

        info!("Server connected to: {}", address);

        info!("Sending request for World Server Token to Region Server");
        let request = WorldUserLoginRequest::new();
        let socket_addr = REGION_SERVER_ADDR.parse().unwrap();
        let key = http_client.send(&socket_addr, request);
        global.add_world_key(user_key, key);
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
