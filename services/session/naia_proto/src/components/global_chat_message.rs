use bevy_ecs::{prelude::Component};

use naia_bevy_shared::{EntityProperty, Property, Replicate};

use social_server_types::{GlobalChatMessageId, Timestamp};

#[derive(Component, Replicate)]
pub struct GlobalChatMessage {
    pub id: Property<GlobalChatMessageId>,
    pub timestamp: Property<Timestamp>,
    pub user_entity: EntityProperty,
    pub message: Property<String>,
}

impl GlobalChatMessage {
    pub fn new(
        id: GlobalChatMessageId,
        timestamp: Timestamp,
        message: &str,
    ) -> Self {
        Self::new_complete(id, timestamp, message.to_string())
    }
}
