use std::time::Duration;

use bevy_ecs::{
    change_detection::ResMut,
    event::{EventReader, EventWriter},
    system::Res,
    system::Resource,
};

use naia_bevy_client::{
    events::{MessageEvents, RequestEvents},
    transport::webrtc::Socket as WebrtcSocket,
    Client, Timer,
};

use asset_loader::{AssetManager, AssetMetadataStore};
use config::TargetEnv;
use filesystem::FileSystemManager;
use kernel::http::HttpClient;
use logging::{info, warn};
use ui_runner::UiManager;

use session_server_naia_proto::{
    channels::{AssetRequestsChannel, PrimaryChannel},
    messages::{LoadAssetRequest, LoadAssetWithData, WorldConnectToken},
};
use world_server_naia_proto::messages::Auth as WorldAuth;

use super::client_markers::{Session, World};
use crate::{
    world::{WorldConnectEvent, WorldRejectEvent, WorldDisconnectEvent}, session::{SessionDisconnectEvent, SessionRejectEvent, SessionConnectEvent},
    asset_cache::{AssetCache, AssetLoadedEvent},
    networked::asset_cache_checker::AssetCacheChecker,
};

type SessionClient<'a> = Client<'a, Session>;
type WorldClient<'a> = Client<'a, World>;

#[derive(Clone, PartialEq, Debug)]
pub enum ConnectionState {
    Disconnected,
    SendingSessionConnect,
    WaitingForSessionConnect,
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
    pub fn handle_session_connect_events(
        client: SessionClient,
        mut session_connect_event_reader: EventReader<SessionConnectEvent>,
        mut connection_manager: ResMut<Self>,
    ) {
        for _ in session_connect_event_reader.read() {
            let Ok(server_address) = client.server_address() else {
                panic!("Shouldn't happen");
            };
            info!(
                "Client connected to session server at addr: {}",
                server_address
            );

            let ConnectionState::WaitingForSessionConnect = &connection_manager.connection_state
            else {
                panic!("Shouldn't happen");
            };

            connection_manager.connection_state = ConnectionState::ConnectedToSession;
        }
    }

    // used as a system
    pub fn handle_session_disconnect_events(
        mut session_disconnect_event_reader: EventReader<SessionDisconnectEvent>,
        mut connection_manager: ResMut<Self>,
    ) {
        for _ in session_disconnect_event_reader.read() {
            warn!("Client disconnected from session server");

            connection_manager.connection_state = ConnectionState::SendingSessionConnect;
        }
    }

    // used as a system
    pub fn handle_session_reject_events(
        mut session_reject_event_reader: EventReader<SessionRejectEvent>,
        mut connection_manager: ResMut<Self>,
    ) {
        for _ in session_reject_event_reader.read() {
            warn!("Client rejected from connecting to the session server");

            connection_manager.connection_state = ConnectionState::SendingSessionConnect;
        }
    }

    // used as a system
    pub fn handle_world_connect_events(
        client: WorldClient,
        mut event_reader: EventReader<WorldConnectEvent>,
        mut connection_manager: ResMut<Self>,
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
    pub fn handle_world_disconnect_events(
        mut event_reader: EventReader<WorldDisconnectEvent>,
        mut connection_manager: ResMut<Self>,
    ) {
        for _ in event_reader.read() {
            info!(
                "Client disconnected from world server",
            );

            let ConnectionState::ConnectedToWorld = &connection_manager.connection_state else {
                panic!("Shouldn't happen .. state is: {:?}", &connection_manager.connection_state);
            };

            connection_manager.connection_state = ConnectionState::ConnectedToSession;
        }
    }

    // used as a system
    pub fn handle_world_reject_events(
        mut world_reject_event_reader: EventReader<WorldRejectEvent>,
        mut _connection_manager: ResMut<Self>,
    ) {
        for _ in world_reject_event_reader.read() {
            warn!("Client rejected from connecting to the world server");

            todo!();
        }
    }

    // used as a system
    pub fn handle_session_message_events(
        mut world_client: WorldClient,
        http_client: Res<HttpClient>,
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
                if let Some(cookies) = http_client.cookie_header_value() {
                    let mut headers = Vec::new();
                    headers.push(("Cookie".to_string(), cookies));
                    world_client.auth_headers(headers);
                }
                let world_server_public_webrtc_url = TargetEnv::gateway_url();
                info!(
                    "connecting to world server: {}",
                    &world_server_public_webrtc_url
                );
                let socket = WebrtcSocket::new(
                    &world_server_public_webrtc_url,
                    world_client.socket_config(),
                );
                world_client.connect(socket);
            }
            for asset_message in events.read::<AssetRequestsChannel, LoadAssetWithData>() {
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
            for (response_send_key, request) in
                events.read::<AssetRequestsChannel, LoadAssetRequest>()
            {
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
        mut connection_manager: ResMut<Self>,
        http_client: Res<HttpClient>,
        mut session_client: SessionClient,
    ) {
        connection_manager.handle_connection_impl(&http_client, &mut session_client);
    }

    fn handle_connection_impl(
        &mut self,
        http_client: &HttpClient,
        session_client: &mut SessionClient,
    ) {
        if self.send_timer.ringing() {
            self.send_timer.reset();
        } else {
            return;
        }

        match &self.connection_state {
            ConnectionState::Disconnected => {
                self.connection_state = ConnectionState::SendingSessionConnect;
            }
            ConnectionState::SendingSessionConnect => {
                // previous below
                self.connection_state = ConnectionState::WaitingForSessionConnect;

                Self::send_session_connect(http_client, session_client);
            }
            ConnectionState::WaitingForSessionConnect => {}
            ConnectionState::ConnectedToSession => {}
            ConnectionState::ConnectedToWorld => {
                info!("world : connected!");
            }
        }
    }

    cfg_if! {
        if #[cfg(feature = "odst")] {
            fn send_session_connect(
                _http_client: &HttpClient,
                session_client: &mut SessionClient
            ) {
                let url = format!("{}://{}:{}", config::PUBLIC_PROTOCOL, config::PUBLIC_IP_ADDR, config::SESSION_SERVER_SIGNAL_PORT);

                info!("connecting to session server: {}", url);
                session_client.auth(session_server_naia_proto::messages::Auth::new("odst"));
                let socket = WebrtcSocket::new(&url, session_client.socket_config());
                session_client.connect(socket);
            }
        } else {
            fn send_session_connect(
                http_client: &HttpClient,
                session_client: &mut SessionClient
            ) {
                let url = TargetEnv::gateway_url();

                info!("connecting to session server: {}", url);
                let socket = WebrtcSocket::new(&url, session_client.socket_config());
                if let Some(cookies) = http_client.cookie_header_value() {
                    let mut headers = Vec::new();
                    headers.push(("Cookie".to_string(), cookies));
                    session_client.auth_headers(headers);
                }
                session_client.connect(socket);
            }
        }
    }
}
