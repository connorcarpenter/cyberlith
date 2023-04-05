use bevy_log::info;

use editor_proto::messages::Auth;
use naia_bevy_client::{transport::webrtc, Client};

pub fn init(mut client: Client) {
    info!("Naia Bevy Client Demo started");

    client.auth(Auth::new("charlie", "12345"));
    let socket = webrtc::Socket::new("http://127.0.0.1:14191", client.socket_config());
    client.connect(socket);
}
