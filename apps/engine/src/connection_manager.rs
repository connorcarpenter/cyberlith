use std::time::Duration;

use bevy_ecs::{event::{EventReader, EventWriter, Event}, system::Resource, change_detection::ResMut};
use bevy_log::info;

use naia_bevy_client::{Timer, events::ConnectEvent};
use asset_render::{AssetManager, AssetMetadataStore};

use bevy_http_client::{HttpClient, ResponseKey};
use config::{ORCHESTRATOR_PORT, PUBLIC_IP_ADDR};
use filesystem::FileSystemManager;
use orchestrator_http_proto::{LoginRequest, LoginResponse};
use session_server_naia_proto::messages::{LoadAssetRequest, LoadAssetWithData, WorldConnectToken};

use crate::{world::{WorldAuth, WorldClient}, session::{SessionMessageEvents, SessionPrimaryChannel, SessionRequestChannel, SessionRequestEvents}, asset::{AssetCache, AssetLoadedEvent}, client_markers::Session, naia::WebrtcSocket, session::{SessionAuth, SessionClient}};

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

#[derive(Event)]
pub struct SessionConnectEvent;

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
    pub fn handle_session_connect_events(
        client: SessionClient,
        mut event_reader: EventReader<ConnectEvent<Session>>,
        mut connection_manager: ResMut<ConnectionManager>,
        mut event_writer: EventWriter<SessionConnectEvent>
    ) {
        for _ in event_reader.read() {
            let Ok(server_address) = client.server_address() else {
                panic!("Shouldn't happen");
            };
            info!(
                "Client connected to session server at addr: {}",
                server_address
            );

            let ConnectionState::ReceivedFromOrchestrator(_) = &connection_manager.connection_state else {
                panic!("Shouldn't happen");
            };

            connection_manager.connection_state = ConnectionState::ConnectedToSession;

            event_writer.send(SessionConnectEvent);
        }
    }

    // used as a system
    pub fn handle_world_connection_event(&mut self) {
        let ConnectionState::ConnectedToSession = &self.connection_state else {
            panic!("Shouldn't happen");
        };

        self.connection_state = ConnectionState::ConnectedToWorld;
    }

    // used as a system
    pub fn handle_session_message_events(
        mut world_client: WorldClient,
        mut asset_cache: ResMut<AssetCache>,
        mut asset_manager: ResMut<AssetManager>,
        mut file_system_manager: ResMut<FileSystemManager>,
        mut metadata_store: ResMut<AssetMetadataStore>,
        mut event_reader: EventReader<SessionMessageEvents>,
        mut asset_loaded_event_writer: EventWriter<AssetLoadedEvent>,
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
            for asset_message in events.read::<SessionPrimaryChannel, LoadAssetWithData>() {
                info!("received Asset Data Message from Session Server! (id: {:?}, etag: {:?})", asset_message.asset_id, asset_message.asset_etag);

                asset_cache.handle_load_asset_with_data_message(&mut asset_manager, &mut asset_loaded_event_writer, &mut file_system_manager, &mut metadata_store, asset_message);
            }
        }
    }

    pub fn handle_session_request_events(
        mut asset_cache: ResMut<AssetCache>,
        mut file_system_manager: ResMut<FileSystemManager>,
        mut metadata_store: ResMut<AssetMetadataStore>,
        mut event_reader: EventReader<SessionRequestEvents>,
    ) {
        for events in event_reader.read() {
            for (response_send_key, request) in events.read::<SessionRequestChannel, LoadAssetRequest>() {
                asset_cache.handle_load_asset_request(&mut file_system_manager, &mut metadata_store, request, response_send_key);
            }
        }
    }

    // used as a system
    pub fn handle_connection(
        mut connection_manager: ResMut<ConnectionManager>,
        mut http_client: ResMut<HttpClient>,
        mut session_client: SessionClient,
    ) {
        connection_manager.handle_connection_impl(&mut http_client, &mut session_client);
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