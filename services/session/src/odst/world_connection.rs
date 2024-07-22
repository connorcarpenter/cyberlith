use naia_bevy_server::{Server, UserKey};

use session_server_naia_proto::{messages::WorldConnectToken, channels::PrimaryChannel};

pub(crate) fn handle_world_connection(
    naia_server: &mut Server,
    user_key: &UserKey,
) {
    let token = WorldConnectToken::new("odst");
    naia_server.send_message::<PrimaryChannel, WorldConnectToken>(&user_key, &token);
}