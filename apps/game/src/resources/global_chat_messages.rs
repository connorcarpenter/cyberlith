use std::collections::BTreeMap;

use bevy_ecs::{system::Resource, entity::Entity};

use game_engine::{social::{GlobalChatMessageId}};

#[derive(Resource)]
pub struct GlobalChatMessages {
    global_chats: BTreeMap<GlobalChatMessageId, Entity>,
}

impl Default for GlobalChatMessages {
    fn default() -> Self {
        Self {
            global_chats: BTreeMap::new(),
        }
    }
}

impl GlobalChatMessages {

    pub fn add_message(&mut self, message_id: GlobalChatMessageId, message_entity: Entity) {

        self.global_chats.insert(message_id, message_entity);

        if self.global_chats.len() > 100 {
            self.global_chats.pop_first();
        }
    }
}