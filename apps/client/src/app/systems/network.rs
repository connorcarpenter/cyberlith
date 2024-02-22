use std::time::Duration;

use bevy_ecs::{
    prelude::Query,
    event::EventReader,
    entity::Entity,
    system::{Commands, ResMut, Resource},
};
use bevy_log::info;

use game_engine::{
    config::{ORCHESTRATOR_PORT, PUBLIC_IP_ADDR},
    http::HttpClient,
    naia::{Timer, WebrtcSocket},
    orchestrator::LoginRequest,
    session::{
        LoadAssetWithData, LoadAssetRequest, SessionAuth, SessionClient, SessionConnectEvent,
        SessionMessageEvents, SessionPrimaryChannel, SessionRequestChannel, SessionRequestEvents,
        WorldConnectToken,
    },
    world::{WorldSpawnEntityEvent, WorldAuth, WorldClient, WorldConnectEvent, AssetEntry, AssetRef, Main, WorldInsertComponentEvents},
    asset::AssetManager,
};
use game_engine::math::{Quat, Vec3};
use game_engine::render::components::{RenderLayers, Transform, Visibility};
use game_engine::world::Alt1;

use crate::app::resources::{asset_store::AssetProcessor, global::Global, asset_store::AssetStore, connection_state::ConnectionState};
use crate::app::systems::scene::ObjectMarker;

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
    mut commands: Commands,
    mut world_client: WorldClient,
    mut asset_store: ResMut<AssetStore>,
    mut asset_manager: ResMut<AssetManager>,
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
        for asset_message in events.read::<SessionPrimaryChannel, LoadAssetWithData>() {
            info!("received Asset Data Message from Session Server! (id: {:?}, etag: {:?})", asset_message.asset_id, asset_message.asset_etag);

            asset_store.handle_load_asset_with_data_message(&mut commands, &mut asset_manager, asset_message);
        }
    }
}

pub fn session_request_events(
    mut commands: Commands,
    mut session_client: SessionClient,
    mut asset_store: ResMut<AssetStore>,
    mut asset_manager: ResMut<AssetManager>,
    mut event_reader: EventReader<SessionRequestEvents>,
) {
    for events in event_reader.read() {
        for (response_send_key, request) in events.read::<SessionRequestChannel, LoadAssetRequest>()
        {
            let response = asset_store.handle_load_asset_request(&mut commands, &mut asset_manager, request);
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

pub fn world_spawn_entity_events(
    mut event_reader: EventReader<WorldSpawnEntityEvent>,
) {
    for events in event_reader.read() {
        info!("received Spawn Entity from World Server! (entity: {:?})", events.entity);
    }
}

// most likely will need to just split this up into individual insert component systems like in editor
pub fn world_insert_component_events(
    mut commands: Commands,
    client: WorldClient,
    mut event_reader: EventReader<WorldInsertComponentEvents>,
    mut asset_store: ResMut<AssetStore>,
    asset_entry_q: Query<&AssetEntry>,
    asset_ref_main_q: Query<&AssetRef<Main>>,
    asset_ref_alt1_q: Query<&AssetRef<Alt1>>,
) {
    for events in event_reader.read() {
        for entity in events.read::<AssetEntry>() {
            let Ok(asset_entry) = asset_entry_q.get(entity) else {
                panic!("Shouldn't happen");
            };
            let asset_id = *asset_entry.asset_id;
            info!("received Asset Entry from World Server! (entity: {:?}, asset_id: {:?})", entity, asset_id);
            asset_store.handle_add_asset_entry(&mut commands, &entity, &asset_id);
        }
        for entity in events.read::<AssetRef<Main>>() {
            insert_asset_ref_events::<Main>(&mut commands, &client, &mut asset_store, &asset_entry_q, &asset_ref_main_q, &entity);

            // add clientside things
            let layer = RenderLayers::layer(0);

            // model
            commands
                .entity(entity)
                // .insert(WalkAnimation {
                //     anim_handle: human_walk_anim_handle,
                //     image_index: 0.0,
                // })
                .insert(
                    Transform::from_translation(Vec3::splat(0.0))
                        .with_rotation(Quat::from_rotation_z(f32::to_radians(0.0))),
                )
                .insert(Visibility::default())
                .insert(ObjectMarker)
                .insert(layer);
        }
        for entity in events.read::<AssetRef<Alt1>>() {
            insert_asset_ref_events::<Alt1>(&mut commands, &client, &mut asset_store, &asset_entry_q, &asset_ref_alt1_q, &entity);
        }
        // .. other components here later
    }
}

fn insert_asset_ref_events<T: AssetProcessor>(
    commands: &mut Commands,
    client: &WorldClient,
    asset_store: &mut AssetStore,
    asset_entry_q: &Query<&AssetEntry>,
    asset_ref_q: &Query<&AssetRef<T>>,
    entity: &Entity
) {
    let Ok(asset_ref) = asset_ref_q.get(*entity) else {
        panic!("Shouldn't happen");
    };
    let Some(asset_entry_entity) = asset_ref.asset_id_entity.get(client) else {
        panic!("Shouldn't happen");
    };
    if let Ok(asset_entry) = asset_entry_q.get(asset_entry_entity) {
        let asset_id = *asset_entry.asset_id;
        asset_store.handle_entity_added_asset_ref::<T>(commands, entity, &asset_id);
    } else {
        // asset entry entity has been replicated, but not the component just yet ...
        asset_store.handle_add_asset_entry_waitlist::<T>(entity, &asset_entry_entity);
    };
}