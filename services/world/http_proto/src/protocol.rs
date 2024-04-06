use bevy_http_shared::Protocol;

use crate::{HeartbeatRequest, WorldConnectRequest};

pub fn protocol() -> Protocol {
    let mut protocol = Protocol::new();
    protocol.add_request::<WorldConnectRequest>();
    protocol.add_request::<HeartbeatRequest>();
    protocol
}
