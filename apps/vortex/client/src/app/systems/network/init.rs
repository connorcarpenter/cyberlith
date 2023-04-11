use bevy_ecs::event::EventReader;
use bevy_log::info;
use naia_bevy_client::{transport::webrtc, Client};
use vortex_proto::messages::Auth;

use crate::app::events::LoginEvent;

pub fn login(mut client: Client, mut login_events: EventReader<LoginEvent>) {
    for login_event in login_events.iter() {
        info!(
            "Connecting to Server with username: {}, password: {}",
            login_event.username, login_event.password
        );
        client.auth(Auth::new(&login_event.username, &login_event.password));
        let socket = webrtc::Socket::new("http://127.0.0.1:14191", client.socket_config());
        client.connect(socket);
    }
}
