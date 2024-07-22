
use bevy_ecs::system::ResMut;

use naia_bevy_server::Server;

use social_server_types::LobbyId;

use crate::resources::lobby_manager::LobbyManager;

pub(crate) fn startup(
    mut naia_server: Server,
    mut lobby_manager: ResMut<LobbyManager>,
) {
    // create starting lobby
    let lobby_id = LobbyId::new(1);
    let lobby_room_key = naia_server.make_room().key();

    lobby_manager.insert_lobby_room_key(lobby_id, lobby_room_key)
}