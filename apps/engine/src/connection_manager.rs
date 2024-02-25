use std::time::Duration;

use bevy_ecs::{system::Resource, change_detection::ResMut};
use bevy_log::info;

use naia_bevy_client::Timer;

use bevy_http_client::{HttpClient, ResponseKey};
use config::{ORCHESTRATOR_PORT, PUBLIC_IP_ADDR};

use orchestrator_http_proto::{LoginRequest, LoginResponse};

use crate::{naia::WebrtcSocket, session::{SessionAuth, SessionClient}};

#[derive(Clone, PartialEq)]
pub enum ConnectionState {
    Disconnected,
    SentToOrchestrator(ResponseKey<LoginResponse>),
    ReceivedFromOrchestrator(LoginResponse),
    ConnectedToSession,
    ConnectedToWorld,
}

#[derive(Resource)]
pub struct ConnectionManager {
    pub connection_state: ConnectionState,
    send_timer: Timer,
}

impl Default for ConnectionManager {
    fn default() -> Self {
        Self {
            connection_state: ConnectionState::Disconnected,
            // TODO: split this out into config var?
            send_timer: Timer::new(Duration::from_millis(5000)),
        }
    }
}

impl ConnectionManager {

    // used as a system
    pub fn handle_connection(
        mut connection_manager: ResMut<ConnectionManager>,
        mut http_client: ResMut<HttpClient>,
        mut session_client: SessionClient,
    ) {
        connection_manager.handle_connection_impl(&mut http_client, &mut session_client);
    }

    // used as a system
    pub fn handle_session_connection_event(&mut self) {
        let ConnectionState::ReceivedFromOrchestrator(_) = &self.connection_state else {
            panic!("Shouldn't happen");
        };

        self.connection_state = ConnectionState::ConnectedToSession;
    }

    // used as a system
    pub fn handle_world_connection_event(&mut self) {
        let ConnectionState::ConnectedToSession = &self.connection_state else {
            panic!("Shouldn't happen");
        };

        self.connection_state = ConnectionState::ConnectedToWorld;
    }

    fn handle_connection_impl(&mut self, http_client: &mut HttpClient, session_client: &mut SessionClient) {
        if self.send_timer.ringing() {
            self.send_timer.reset();
        } else {
            return;
        }

        match &self.connection_state {
            ConnectionState::Disconnected => {
                info!("sending to orchestrator..");
                let request = LoginRequest::new("charlie", "12345");
                let key = http_client.send(PUBLIC_IP_ADDR, ORCHESTRATOR_PORT, request);
                self.connection_state = ConnectionState::SentToOrchestrator(key);
            }
            ConnectionState::SentToOrchestrator(key) => {
                if let Some(result) = http_client.recv(key) {
                    match result {
                        Ok(response) => {
                            info!(
                            "received from orchestrator: (webrtc url: {:?}, token: {:?})",
                            response.session_server_public_webrtc_url, response.token
                        );
                            self.connection_state =
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
                            self.connection_state = ConnectionState::Disconnected;
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
}