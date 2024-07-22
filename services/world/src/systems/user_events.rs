use std::{collections::HashMap, net::SocketAddr};

use bevy_ecs::{
    change_detection::{Mut, ResMut},
    entity::Entity,
    event::EventReader,
    prelude::{Query, Resource, World},
    system::{Commands, Res, SystemState},
};

use naia_bevy_server::{
    CommandsExt,
    events::{AuthEvents, ConnectEvent, DisconnectEvent, ErrorEvent, TickEvent},
    Server, transport::webrtc, UserKey,
};

use bevy_http_client::HttpClient;
use config::{
    PUBLIC_IP_ADDR, PUBLIC_PROTOCOL, SELF_BINDING_ADDR, WORLD_SERVER_SIGNAL_PORT,
    WORLD_SERVER_WEBRTC_PORT,
};
use logging::{info, warn};
use world_server_naia_proto::{
    components::{Alt1, AssetEntry, AssetRef, Main},
    messages::Auth,
};

use crate::resources::{asset_manager::{AssetCatalog, AssetCommandsExt, AssetManager}, lobby_manager::LobbyManager, user_manager::UserManager, world_instance::WorldInstance};

pub fn init(mut commands: Commands, mut server: Server) {
    info!("World Naia Server starting up");

    // set up server
    let server_addresses = webrtc::ServerAddrs::new(
        // IP Address to listen on for WebRTC signaling
        SocketAddr::new(SELF_BINDING_ADDR.parse().unwrap(), WORLD_SERVER_SIGNAL_PORT),
        // IP Address to listen on for UDP WebRTC data channels
        SocketAddr::new(SELF_BINDING_ADDR.parse().unwrap(), WORLD_SERVER_WEBRTC_PORT),
        // The public WebRTC IP address to advertise
        format!(
            "{}://{}:{}",
            PUBLIC_PROTOCOL, PUBLIC_IP_ADDR, WORLD_SERVER_WEBRTC_PORT
        )
        .as_str(),
    );
    let socket = webrtc::Socket::new(&server_addresses, server.socket_config());
    server.listen(socket);

    // set up global
    #[cfg(not(feature = "odst"))]
    let instance_secret = random::generate_random_string(16);

    #[cfg(feature = "odst")]
    let instance_secret = "odst".to_string();

    commands.insert_resource(WorldInstance::new(&instance_secret));
}

pub fn auth_events(
    mut user_manager: ResMut<UserManager>,
    mut server: Server,
    mut event_reader: EventReader<AuthEvents>,
) {
    for events in event_reader.read() {
        for (user_key, auth) in events.read::<Auth>() {
            if let Some(user_data) = user_manager.spend_login_token(&auth.login_token) {
                info!(
                    "Accepted connection. User Id: {:?}, Token: {}",
                    user_data.user_id, auth.login_token
                );

                user_manager.add_user(&user_key, user_data);

                // Accept incoming connection
                server.accept_connection(&user_key);
            } else {
                warn!("Rejected connection. Token: {}", auth.login_token);

                // Reject incoming connection
                server.reject_connection(&user_key);
            }
        }
    }
}

pub fn connect_events(
    mut commands: Commands,
    mut server: Server,
    lobby_manager: Res<LobbyManager>,
    user_manager: Res<UserManager>,
    mut asset_manager: ResMut<AssetManager>,
    mut event_reader: EventReader<ConnectEvent>,
) {
    for ConnectEvent(user_key) in event_reader.read() {
        let address = server.user(user_key).address();

        info!("Server connected to: {}", address);

        // add user to main room
        let lobby_id = user_manager.get_user_lobby_id(user_key).unwrap();
        let lobby_room_key = lobby_manager.lobby_room_key(&lobby_id).unwrap();
        server.room_mut(&lobby_room_key).add_user(&user_key);

        // give user an entity
        let entity = commands
            // Spawn new Entity
            .spawn_empty()
            // MUST call this to begin replication
            .enable_replication(&mut server)
            // insert asset ref
            .insert_asset::<Main>(
                &mut asset_manager,
                &mut server,
                AssetCatalog::HumanModel.into(),
            )
            .insert_asset::<Alt1>(
                &mut asset_manager,
                &mut server,
                AssetCatalog::HumanWalk.into(),
            )
            // return Entity id
            .id();

        // add entity to main room
        server.room_mut(&lobby_room_key).add_entity(&entity);

        // TODO: need to clean up this entity on disconnect

        // register user
        asset_manager.register_user(&mut server, user_key);
    }
}

pub fn disconnect_events(
    mut asset_manager: ResMut<AssetManager>,
    mut event_reader: EventReader<DisconnectEvent>,
) {
    for DisconnectEvent(user_key, user) in event_reader.read() {
        info!("Server disconnected from: {:?}", user.address());

        asset_manager.deregister_user(user_key);
    }
}

pub fn error_events(mut event_reader: EventReader<ErrorEvent>) {
    for ErrorEvent(error) in event_reader.read() {
        info!("Server Error: {:?}", error);
    }
}

#[derive(Resource)]
struct CachedTickEventsState {
    event_state: SystemState<EventReader<'static, 'static, TickEvent>>,
}

pub fn tick_events_startup(world: &mut World) {
    let event_state: SystemState<EventReader<TickEvent>> = SystemState::new(world);
    world.insert_resource(CachedTickEventsState { event_state });
}

pub fn tick_events(world: &mut World) {
    let mut has_ticked = false;

    world.resource_scope(
        |world, mut events_reader_state: Mut<CachedTickEventsState>| {
            let mut events_reader = events_reader_state.event_state.get_mut(world);

            for _tick_event in events_reader.read() {
                has_ticked = true;
            }
        },
    );

    if !has_ticked {
        return;
    }

    // get all scope checks from server
    let mut scope_checks = Vec::new();

    {
        let mut system_state: SystemState<Server> = SystemState::new(world);
        let server = system_state.get_mut(world);

        // Update scopes of entities
        for (room_key, user_key, entity) in server.scope_checks() {
            let in_scope = server.user_scope(&user_key).has(&entity);
            scope_checks.push((room_key, user_key, entity, in_scope));
        }
    }

    if scope_checks.is_empty() {
        return;
    }

    // calculate all updates to scope needed
    let mut scope_actions: HashMap<(UserKey, Entity), bool> = HashMap::new();

    for (_room_key, user_key, entity, in_scope) in scope_checks {

        // TODO: assess scope logic here ..
        // right now, everything not in scope is added to user
        // however this will change later

        if !in_scope {
            info!(
                "Entity out of scope: {:?}, should be added to user.",
                entity
            );
            scope_actions.insert((user_key, entity), true);
        }
    }

    if scope_actions.is_empty() {
        return;
    }

    // actually update all scopes
    {
        let mut system_state: SystemState<Server> = SystemState::new(world);
        let mut server = system_state.get_mut(world);

        for ((user_key, entity), include) in scope_actions.iter() {
            if *include {
                if server.user_scope(&user_key).has(&entity) {
                    panic!("Entity already in scope, shouldn't happen");
                }
                info!("Adding entity to user scope: {:?}", entity);
                server.user_scope_mut(&user_key).include(&entity);
            } else {
                if !server.user_scope(&user_key).has(&entity) {
                    panic!("Entity already out of scope, shouldn't happen");
                }
                info!("Removing entity from user scope: {:?}", entity);
                server.user_scope_mut(&user_key).exclude(&entity);
            }
        }
    }

    // determine if any entities that have gone into or out of scope have AssetRef components
    let mut asset_id_entity_actions = Vec::new();

    {
        let mut system_state: SystemState<(
            Server,
            Query<&AssetEntry>,
            Query<&AssetRef<Main>>,
            Query<&AssetRef<Alt1>>,
        )> = SystemState::new(world);
        let (server, asset_entry_q, asset_ref_main_q, asset_ref_alt1_q) =
            system_state.get_mut(world);

        for ((user_key, entity), include) in scope_actions.iter() {
            // determine if entity has any AssetRef components
            info!("Checking entity for AssetRefs: {:?}", entity);

            // AssetRef<Main>
            if let Ok(asset_ref) = asset_ref_main_q.get(*entity) {
                let asset_id_entity = asset_ref.asset_id_entity.get(&server).unwrap();
                let asset_id = *asset_entry_q.get(asset_id_entity).unwrap().asset_id;

                info!(
                    "entity {:?} has AssetRef<Main>(asset_id: {:?})",
                    entity, asset_id
                );

                asset_id_entity_actions.push((*user_key, asset_id, *include));
            }
            // AssetRef<Alt1>
            if let Ok(asset_ref) = asset_ref_alt1_q.get(*entity) {
                let asset_id_entity = asset_ref.asset_id_entity.get(&server).unwrap();
                let asset_id = *asset_entry_q.get(asset_id_entity).unwrap().asset_id;

                info!(
                    "entity {:?} has AssetRef<Alt1>(asset_id: {:?})",
                    entity, asset_id
                );

                asset_id_entity_actions.push((*user_key, asset_id, *include));
            }
            // this is unecessary, just for logging
            if let Ok(asset_entry) = asset_entry_q.get(*entity) {
                let asset_id = *asset_entry.asset_id;

                info!(
                    "entity {:?} has AssetEntry(asset_id: {:?})",
                    entity, asset_id
                );
            }

            // TODO: put other AssetRef<Marker> components here .. also could clean this up somehow??
        }
    }

    if asset_id_entity_actions.is_empty() {
        return;
    }

    // update asset id entities in asset manager
    {
        let mut system_state: SystemState<(
            Server,
            Res<WorldInstance>,
            Res<UserManager>,
            ResMut<AssetManager>,
            ResMut<HttpClient>,
        )> = SystemState::new(world);
        let (
            mut server,
            world_instance,
            user_manager,
            mut asset_manager,
            mut http_client
        ) = system_state.get_mut(world);

        asset_manager.handle_scope_actions(
            &mut server,
            &world_instance,
            &user_manager,
            &mut http_client,
            asset_id_entity_actions,
        );
    }
}
