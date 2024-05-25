use std::collections::BTreeMap;

use bevy_ecs::system::Resource;

use game_engine::{logging::info, auth::UserId, social::{Timestamp, GlobalChatMessageId}};

#[derive(Resource)]
pub struct GlobalChatMessages {
    global_chats: BTreeMap<GlobalChatMessageId, (Timestamp, UserId, String)>,
}

impl Default for GlobalChatMessages {
    fn default() -> Self {
        Self {
            global_chats: BTreeMap::new(),
        }
    }
}

impl GlobalChatMessages {
    pub fn add_message(&mut self, message_id: GlobalChatMessageId, timestamp: Timestamp, user_id: UserId, message: &str) {
        info!("added global message: [user_id({:?}) | {:?} | {:?}]", user_id, timestamp, message);
        self.global_chats.insert(message_id, (timestamp, user_id, message.to_string()));

        if self.global_chats.len() > 100 {
            self.global_chats.pop_first();
        }
    }
}