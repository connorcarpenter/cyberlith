use bevy_ecs::prelude::Component;

use naia_bevy_shared::{Property, Replicate};

use auth_server_types::UserId;

#[derive(Component, Replicate)]
pub struct GlobalChatMessage {
    pub timestamp: Property<u16>, // this should be monotonically increasing ... TODO: replace with an actual timestamp
    pub user_id: Property<UserId>,
    pub message: Property<String>,
}

impl GlobalChatMessage {
    pub fn new(timestamp: u16, user_id: UserId, message: &str) -> Self {
        Self::new_complete(timestamp, user_id, message.to_string())
    }
}
