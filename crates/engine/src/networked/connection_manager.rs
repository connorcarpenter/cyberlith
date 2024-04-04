use std::time::Duration;

use bevy_ecs::system::Res;
use bevy_ecs::{
    change_detection::ResMut,
    event::{Event, EventReader, EventWriter},
    system::Resource,
};
use bevy_log::info;

use naia_bevy_client::{
    events::{ConnectEvent, MessageEvents, RequestEvents},
    transport::webrtc::Socket as WebrtcSocket,
    Client, Timer,
};

use asset_loader::{AssetManager, AssetMetadataStore};
use bevy_http_client::{HttpClient, ResponseKey};
use config::{GATEWAY_PORT, PUBLIC_IP_ADDR};
use filesystem::FileSystemManager;
use ui_runner::UiManager;

use gateway_http_proto::{SessionConnectRequest, SessionConnectResponse};
use session_server_naia_proto::{
    channels::{PrimaryChannel, RequestChannel},
    messages::{Auth as SessionAuth, LoadAssetRequest, LoadAssetWithData, WorldConnectToken},
};
use world_server_naia_proto::messages::Auth as WorldAuth;

use crate::asset_cache::{AssetCache, AssetLoadedEvent};
use crate::networked::asset_cache_checker::AssetCacheChecker;

use super::client_markers::{Session, World};

type SessionClient<'a> = Client<'a, Session>;
type WorldClient<'a> = Client<'a, World>;

#[derive(Clone, PartialEq)]
pub enum ConnectionState {
    Disconnected,
    SentToGateway(ResponseKey<SessionConnectResponse>),
    ReceivedFromGateway(SessionConnectResponse),
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
        mut event_writer: EventWriter<SessionConnectEvent>,
    ) {
        for _ in event_reader.read() {
            let Ok(server_address) = client.server_address() else {
                panic!("Shouldn't happen");
            };
            info!(
                "Client connected to session server at addr: {}",
                server_address
            );

            let ConnectionState::ReceivedFromGateway(_) = &connection_manager.connection_state
            else {
                panic!("Shouldn't happen");
            };

            connection_manager.connection_state = ConnectionState::ConnectedToSession;

            event_writer.send(SessionConnectEvent);
        }
    }

    // used as a system
    pub fn handle_world_connect_events(
        client: WorldClient,
        mut event_reader: EventReader<ConnectEvent<World>>,
        mut connection_manager: ResMut<ConnectionManager>,
    ) {
        for _ in event_reader.read() {
            let Ok(server_address) = client.server_address() else {
                panic!("Shouldn't happen");
            };
            info!(
                "Client connected to world server at addr: {}",
                server_address
            );

            let ConnectionState::ConnectedToSession = &connection_manager.connection_state else {
                panic!("Shouldn't happen");
            };

            connection_manager.connection_state = ConnectionState::ConnectedToWorld;
        }
    }

    // used as a system
    pub fn handle_session_message_events(
        mut world_client: WorldClient,
        mut asset_cache: ResMut<AssetCache>,
        mut asset_manager: ResMut<AssetManager>,
        mut ui_manager: ResMut<UiManager>,
        mut file_system_manager: ResMut<FileSystemManager>,
        mut metadata_store: ResMut<AssetMetadataStore>,
        mut event_reader: EventReader<MessageEvents<Session>>,
        mut asset_loaded_event_writer: EventWriter<AssetLoadedEvent>,
    ) {
        for events in event_reader.read() {
            for token in events.read::<PrimaryChannel, WorldConnectToken>() {
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
            for asset_message in events.read::<PrimaryChannel, LoadAssetWithData>() {
                info!(
                    "received Asset Data Message from Session Server! (id: {:?}, etag: {:?})",
                    asset_message.asset_id, asset_message.asset_etag
                );

                let LoadAssetWithData {
                    asset_id,
                    asset_etag,
                    asset_type,
                    asset_data,
                } = asset_message;

                asset_cache.handle_load_asset_with_data_message(
                    &mut asset_manager,
                    &mut ui_manager,
                    &mut asset_loaded_event_writer,
                    &mut file_system_manager,
                    &mut metadata_store,
                    asset_id,
                    asset_etag,
                    asset_type,
                    asset_data,
                );
            }
        }
    }

    pub fn handle_session_request_events(
        asset_cache: Res<AssetCache>,
        mut asset_cache_checker: ResMut<AssetCacheChecker>,
        mut file_system_manager: ResMut<FileSystemManager>,
        mut metadata_store: ResMut<AssetMetadataStore>,
        mut event_reader: EventReader<RequestEvents<Session>>,
    ) {
        for events in event_reader.read() {
            for (response_send_key, request) in events.read::<RequestChannel, LoadAssetRequest>() {
                asset_cache_checker.handle_load_asset_request(
                    &asset_cache,
                    &mut file_system_manager,
                    &mut metadata_store,
                    request,
                    response_send_key,
                );
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

    fn handle_connection_impl(
        &mut self,
        http_client: &mut HttpClient,
        session_client: &mut SessionClient,
    ) {
        if self.send_timer.ringing() {
            self.send_timer.reset();
        } else {
            return;
        }

        match &self.connection_state {
            ConnectionState::Disconnected => {
                info!("sending to gateway..");
                let request = SessionConnectRequest::new("charlie", "12345");
                let key = http_client.send(PUBLIC_IP_ADDR, GATEWAY_PORT, request);
                self.connection_state = ConnectionState::SentToGateway(key);
            }
            ConnectionState::SentToGateway(key) => {
                if let Some(result) = http_client.recv(key) {
                    match result {
                        Ok(response) => {
                            info!(
                                "received from gateway: (webrtc url: {:?}, token: {:?})",
                                response.session_server_public_webrtc_url, response.token
                            );
                            self.connection_state =
                                ConnectionState::ReceivedFromGateway(response.clone());

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
                            info!("resending to gateway..");
                            self.connection_state = ConnectionState::Disconnected;
                        }
                    }
                }
            }
            ConnectionState::ReceivedFromGateway(_response) => {
                // waiting for connect event ..
            }
            ConnectionState::ConnectedToSession => {}
            ConnectionState::ConnectedToWorld => {
                info!("world : connected!");
            }
        }
    }
}
