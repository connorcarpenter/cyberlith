use bevy_http_shared::Protocol;

use crate::{ConnectAssetServerRequest, ConnectSocialServerRequest, DisconnectAssetServerRequest, DisconnectSocialServerRequest, HeartbeatRequest, IncomingUserRequest, SocialPatchGlobalChatMessagesRequest, SocialPatchMatchLobbiesRequest, SocialPatchUsersRequest, SocialWorldConnectRequest, UserAssetIdRequest};

pub fn protocol() -> Protocol {
    let mut protocol = Protocol::new();
    protocol.add_request::<IncomingUserRequest>();
    protocol.add_request::<HeartbeatRequest>();

    protocol.add_request::<ConnectAssetServerRequest>();
    protocol.add_request::<DisconnectAssetServerRequest>();
    protocol.add_request::<UserAssetIdRequest>();

    protocol.add_request::<ConnectSocialServerRequest>();
    protocol.add_request::<DisconnectSocialServerRequest>();

    protocol.add_request::<SocialPatchUsersRequest>();
    protocol.add_request::<SocialPatchGlobalChatMessagesRequest>();
    protocol.add_request::<SocialPatchMatchLobbiesRequest>();
    protocol.add_request::<SocialWorldConnectRequest>();

    protocol
}
