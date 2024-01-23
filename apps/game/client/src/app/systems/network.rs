use std::{net::SocketAddr, str::FromStr, time::Duration};

use bevy_ecs::{
    system::{ResMut, Resource},
    event::EventReader,
};
use bevy_log::info;

use game_engine::{
    http::HttpClient,
    naia::{Timer, WebrtcSocket},
    session::{SessionAuth, SessionClient, SessionConnectEvent},
    orchestrator::LoginRequest};

use crate::app::{connection_state::ConnectionState, global::Global};

// ApiTimer
#[derive(Resource)]
pub struct ApiTimer(pub Timer);

impl Default for ApiTimer {
    fn default() -> Self {
        Self(Timer::new(Duration::from_millis(1000)))
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
            let socket_addr = SocketAddr::from_str("127.0.0.1:14197").unwrap();
            let key = http_client.send(&socket_addr, request);
            global.connection_state = ConnectionState::SentToOrchestrator(key);
        }
        ConnectionState::SentToOrchestrator(key) => {
            if let Some(result) = http_client.recv(key) {
                match result {
                    Ok(response) => {
                        info!("received from orchestrator: (addr: {:?}, token: {:?})", response.session_server_addr, response.token);
                        global.connection_state = ConnectionState::ReceivedFromOrchestrator(response.clone());

                        session_client.auth(SessionAuth::new(&response.token));
                        let server_session_url = format!("http://{}:{}", response.session_server_addr.inner().ip(), response.session_server_addr.inner().port());
                        info!("connecting to session server: {}", server_session_url);
                        let socket = WebrtcSocket::new(
                            &server_session_url, //"http://127.0.0.1:14191",
                            session_client.socket_config()
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
        ConnectionState::Connected => {
            info!("Connected!")
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
        info!("Client connected to session server at addr: {}", server_address);

        let ConnectionState::ReceivedFromOrchestrator(_) = &global.connection_state else {
            panic!("Shouldn't happen");
        };

        global.connection_state = ConnectionState::Connected;
    }
}
