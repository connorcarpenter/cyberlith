use bevy_ecs::prelude::Component;

use naia_bevy_shared::{Property, Replicate};

use auth_server_types::UserId;
use social_server_types::{GlobalChatMessageId, Timestamp};

#[derive(Component, Replicate)]
pub struct GlobalChatMessage {
    pub id: Property<GlobalChatMessageId>,
    pub timestamp: Property<Timestamp>,
    pub user_id: Property<UserId>,
    pub message: Property<String>,
}

impl GlobalChatMessage {
    pub fn new(
        id: GlobalChatMessageId,
        timestamp: Timestamp,
        user_id: UserId,
        message: &str,
    ) -> Self {
        Self::new_complete(id, timestamp, user_id, message.to_string())
    }
}
