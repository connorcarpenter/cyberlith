use bevy_http_shared::Protocol;

use crate::IncomingUserRequest;

pub fn protocol() -> Protocol {
    let mut protocol = Protocol::new();
    protocol.add_request::<IncomingUserRequest>();
    protocol
}