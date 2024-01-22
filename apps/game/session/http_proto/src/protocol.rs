use bevy_http_shared::Protocol;

use crate::LoginRequest;

pub fn protocol() -> Protocol {
    let mut protocol = Protocol::new();
    protocol.add_request::<LoginRequest>();
    protocol
}