use bevy_http_shared::Protocol;

use crate::{HeartbeatRequest, IncomingUserRequest};

pub fn protocol() -> Protocol {
    let mut protocol = Protocol::new();
    protocol.add_request::<IncomingUserRequest>();
    protocol.add_request::<HeartbeatRequest>();
    protocol
}
