use bevy_ecs::system::Commands;

use naia_bevy_server::{Server, UserKey};

use auth_server_types::UserId;
use bevy_http_client::HttpClient;
use session_server_naia_proto::{channels::PrimaryChannel, messages::WorldConnectToken};
use social_server_types::LobbyId;

use crate::{social::SocialManager, user::UserManager, world::WorldManager};

pub(crate) fn handle_world_connection(
    commands: &mut Commands,
    naia_server: &mut Server,
    http_client: &mut HttpClient,
    world_manager: &mut WorldManager,
    user_manager: &mut UserManager,
    social_manager: &mut SocialManager,
    user_key: &UserKey,
) {
    // test data
    let world_instance_secret = "odst";
    let lobby_id = LobbyId::new(1);
    // test data

    let user_id = user_manager.user_key_to_id(user_key).unwrap();

    setup_lobby(
        commands,
        naia_server,
        http_client,
        user_manager,
        social_manager,
        &lobby_id,
        &user_id,
    );

    world_manager.world_set_user_connected(&user_id, user_key, world_instance_secret);
    // store world instance secret with user key
    user_manager.user_set_world_connected(user_key, world_instance_secret);

    let user_entity = user_manager.get_user_entity(&user_id).unwrap();
    let lobby_room_key = social_manager
        .lobby_manager
        .get_lobby_room_key(&lobby_id)
        .unwrap();
    naia_server
        .room_mut(&lobby_room_key)
        .add_entity(&user_entity);

    let user_id: u64 = user_id.into();
    let token = format!("odst:{}", user_id);
    let token = WorldConnectToken::new(&token);
    naia_server.send_message::<PrimaryChannel, WorldConnectToken>(&user_key, &token);
}

fn setup_lobby(
    // mut commands: Commands,
    // mut naia_server: Server,
    // mut http_client: ResMut<HttpClient>,
    // mut user_manager: ResMut<UserManager>,
    // mut social_manager: ResMut<SocialManager>,
    commands: &mut Commands,
    naia_server: &mut Server,
    http_client: &mut HttpClient,
    user_manager: &mut UserManager,
    social_manager: &mut SocialManager,
    lobby_id: &LobbyId,
    user_id: &UserId,
) {
    if user_id != &UserId::new(1) {
        return;
    }

    let main_menu_room_key = social_manager.global_room_key().unwrap();
    let match_name = "odst";

    if social_manager.lobby_manager.has_lobby(lobby_id) {
        panic!("Lobby already exists");
    }

    social_manager.lobby_manager.create_lobby(
        commands,
        naia_server,
        http_client,
        user_manager,
        &main_menu_room_key,
        lobby_id,
        match_name,
        user_id,
    );
}
