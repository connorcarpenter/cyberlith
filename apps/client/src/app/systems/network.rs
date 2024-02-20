use std::time::Duration;

use bevy_ecs::{
    event::EventReader,
    system::{ResMut, Resource},
};
use bevy_log::info;

use crate::app::resources::asset_cache::AssetCache;
use game_engine::asset::AssetManager;
use game_engine::{
    config::{ORCHESTRATOR_PORT, PUBLIC_IP_ADDR},
    http::HttpClient,
    naia::{Timer, WebrtcSocket},
    orchestrator::LoginRequest,
    session::{
        AssetDataMessage, AssetEtagRequest, SessionAuth, SessionClient, SessionConnectEvent,
        SessionMessageEvents, SessionPrimaryChannel, SessionRequestChannel, SessionRequestEvents,
        WorldConnectToken,
    },
    world::{WorldAuth, WorldClient, WorldConnectEvent},
};

use crate::app::resources::connection_state::ConnectionState;
use crate::app::resources::global::Global;

// ApiTimer
#[derive(Resource)]
pub struct ApiTimer(pub Timer);

impl Default for ApiTimer {
    fn default() -> Self {
        Self(Timer::new(Duration::from_millis(5000)))
    }
}

pub fn handle_connection(
    mut global: ResMut<Global>,
    mut timer: ResMut<ApiTimer>,
    mut http_client: ResMut<HttpClient>,
    mut session_client: SessionClient,
) {
    if timer.0.ringing() {
        timer.0.reset();
    } else {
        return;
    }

    match &global.connection_state {
        ConnectionState::Disconnected => {
            info!("sending to orchestrator..");
            let request = LoginRequest::new("charlie", "12345");
            let key = http_client.send(PUBLIC_IP_ADDR, ORCHESTRATOR_PORT, request);
            global.connection_state = ConnectionState::SentToOrchestrator(key);
        }
        ConnectionState::SentToOrchestrator(key) => {
            if let Some(result) = http_client.recv(key) {
                match result {
                    Ok(response) => {
                        info!(
                            "received from orchestrator: (webrtc url: {:?}, token: {:?})",
                            response.session_server_public_webrtc_url, response.token
                        );
                        global.connection_state =
                            ConnectionState::ReceivedFromOrchestrator(response.clone());

                        session_client.auth(SessionAuth::new(&response.token));
                        info!(
                            "connecting to session server: {}",
                            response.session_server_public_webrtc_url
                        );
                        let socket = WebrtcSocket::new(
                            &response.session_server_public_webrtc_url,
                            session_client.socket_config(),
                        );
                        session_client.connect(socket);
                    }
                    Err(_) => {
                        info!("resending to orchestrator..");
                        global.connection_state = ConnectionState::Disconnected;
                    }
                }
            }
        }
        ConnectionState::ReceivedFromOrchestrator(_response) => {
            // waiting for connect event ..
        }
        ConnectionState::ConnectedToSession => {}
        ConnectionState::ConnectedToWorld => {
            info!("world : connected!");
        }
    }
}

pub fn session_connect_events(
    client: SessionClient,
    mut event_reader: EventReader<SessionConnectEvent>,
    mut global: ResMut<Global>,
) {
    for _ in event_reader.read() {
        let Ok(server_address) = client.server_address() else {
            panic!("Shouldn't happen");
        };
        info!(
            "Client connected to session server at addr: {}",
            server_address
        );

        let ConnectionState::ReceivedFromOrchestrator(_) = &global.connection_state else {
            panic!("Shouldn't happen");
        };

        global.connection_state = ConnectionState::ConnectedToSession;
    }
}

pub fn session_message_events(
    mut world_client: WorldClient,
    mut asset_cache: ResMut<AssetCache>,
    mut event_reader: EventReader<SessionMessageEvents>,
) {
    for events in event_reader.read() {
        for token in events.read::<SessionPrimaryChannel, WorldConnectToken>() {
            info!("received World Connect Token from Session Server!");

            world_client.auth(WorldAuth::new(&token.login_token));
            info!(
                "connecting to world server: {}",
                token.world_server_public_webrtc_url
            );
            let socket = WebrtcSocket::new(
                &token.world_server_public_webrtc_url,
                world_client.socket_config(),
            );
            world_client.connect(socket);
        }
        for asset_message in events.read::<SessionPrimaryChannel, AssetDataMessage>() {
            info!("received Asset Data Message from Session Server! (id: {:?}, etag: {:?})", asset_message.asset_id, asset_message.asset_etag);

            asset_cache.handle_asset_data_message(asset_message);
        }
    }
}

pub fn session_request_events(
    mut session_client: SessionClient,
    mut asset_cache: ResMut<AssetCache>,
    mut event_reader: EventReader<SessionRequestEvents>,
) {
    for events in event_reader.read() {
        for (response_send_key, request) in events.read::<SessionRequestChannel, AssetEtagRequest>()
        {
            info!("received Asset Etag Request from Session Server!");

            let response = asset_cache.handle_etag_request(request);
            let response_result = session_client.send_response(&response_send_key, &response);
            if !response_result {
                panic!("Failed to send response to session server");
            }
        }
    }
}

pub fn world_connect_events(
    client: WorldClient,
    mut event_reader: EventReader<WorldConnectEvent>,
    mut global: ResMut<Global>,
) {
    for _ in event_reader.read() {
        let Ok(server_address) = client.server_address() else {
            panic!("Shouldn't happen");
        };
        info!(
            "Client connected to world server at addr: {}",
            server_address
        );

        let ConnectionState::ConnectedToSession = &global.connection_state else {
            panic!("Shouldn't happen");
        };

        global.connection_state = ConnectionState::ConnectedToWorld;
    }
}
