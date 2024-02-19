use bevy_http_shared::Protocol;

use crate::{AddedAssetIdRequest, ConnectAssetServerRequest, DisconnectAssetServerRequest, HeartbeatRequest, IncomingUserRequest, RemovedAssetIdRequest};

pub fn protocol() -> Protocol {
    let mut protocol = Protocol::new();
    protocol.add_request::<IncomingUserRequest>();
    protocol.add_request::<HeartbeatRequest>();
    protocol.add_request::<ConnectAssetServerRequest>();
    protocol.add_request::<DisconnectAssetServerRequest>();
    protocol.add_request::<AddedAssetIdRequest>();
    protocol.add_request::<RemovedAssetIdRequest>();
    protocol
}