use bevy_ecs::prelude::Component;

use naia_bevy_shared::{EntityProperty, Property, Replicate};

use social_server_types::{MessageId, Timestamp};

#[derive(Component, Replicate)]
pub struct ChatMessage {
    pub id: Property<MessageId>,
    pub timestamp: Property<Timestamp>,
    pub owner_user_entity: EntityProperty,
    pub message: Property<String>,
}

impl ChatMessage {
    pub fn new(id: MessageId, timestamp: Timestamp, message: &str) -> Self {
        Self::new_complete(id, timestamp, message.to_string())
    }
}
