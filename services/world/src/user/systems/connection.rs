
use bevy_ecs::{
    change_detection::ResMut,
    event::EventReader,
    system::{Commands, Res},
};

use naia_bevy_server::{
    CommandsExt,
    events::{AuthEvents, ConnectEvent, DisconnectEvent},
    Server,
};

use logging::{info, warn};
use world_server_naia_proto::{
    components::{Alt1, Main},
    messages::Auth,
};

use crate::{asset::{AssetCatalog, AssetCommandsExt, AssetManager}, social::LobbyManager, user::UserManager};

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