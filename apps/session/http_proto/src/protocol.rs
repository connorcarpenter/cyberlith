use bevy_http_shared::Protocol;

use crate::{UserAssetIdRequest, ConnectAssetServerRequest, DisconnectAssetServerRequest, HeartbeatRequest, IncomingUserRequest};

pub fn protocol() -> Protocol {
    let mut protocol = Protocol::new();
    protocol.add_request::<IncomingUserRequest>();
    protocol.add_request::<HeartbeatRequest>();
    protocol.add_request::<ConnectAssetServerRequest>();
    protocol.add_request::<DisconnectAssetServerRequest>();
    protocol.add_request::<UserAssetIdRequest>();
    protocol
}