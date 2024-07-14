use auth_server_types::UserId;
use naia_serde::SerdeInternal as Serde;

use bevy_http_shared::{ApiRequest, ApiResponse, Method};
use social_server_types::{MessageId, Timestamp};

// Request
#[derive(Serde, PartialEq, Clone)]
pub struct SocialPatchGlobalChatMessagesRequest {
    social_secret: String,
    new_messages: Vec<(MessageId, Timestamp, UserId, String)>,
}

impl SocialPatchGlobalChatMessagesRequest {
    pub fn new(
        social_secret: &str,
        new_messages: Vec<(MessageId, Timestamp, UserId, String)>,
    ) -> Self {
        Self {
            social_secret: social_secret.to_string(),
            new_messages,
        }
    }

    pub fn social_secret(&self) -> &str {
        &self.social_secret
    }

    pub fn new_messages(&self) -> &Vec<(MessageId, Timestamp, UserId, String)> {
        &self.new_messages
    }
}

// Response
#[derive(Serde, PartialEq, Clone)]
pub struct SocialPatchGlobalChatMessagesResponse;

// Traits
impl ApiRequest for SocialPatchGlobalChatMessagesRequest {
    type Response = SocialPatchGlobalChatMessagesResponse;

    fn name() -> &'static str {
        "SocialPatchGlobalChatMessagesRequest"
    }

    fn method() -> Method {
        Method::Post
    }

    fn path() -> &'static str {
        "social_patch_global_chat_messages"
    }
}

impl ApiResponse for SocialPatchGlobalChatMessagesResponse {
    fn name() -> &'static str {
        "SocialPatchGlobalChatMessagesResponse"
    }
}
