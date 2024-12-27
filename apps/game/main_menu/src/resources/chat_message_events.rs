use std::collections::HashMap;

use bevy_ecs::{entity::Entity, event::EventReader, prelude::Resource};

use game_app_network::session::{
    components::{ChatMessage, ChatMessageGlobal, ChatMessageLocal},
    SessionInsertComponentEvent,
};

struct ChatMessageRecord {
    has_message: Option<()>,
    is_global: Option<bool>,
}

impl ChatMessageRecord {
    pub fn new() -> Self {
        Self {
            has_message: None,
            is_global: None,
        }
    }

    pub fn recv_message(&mut self) {
        self.has_message = Some(());
    }

    pub fn recv_global(&mut self) {
        self.is_global = Some(true);
    }

    pub fn recv_local(&mut self) {
        self.is_global = Some(false);
    }

    pub fn is_done(&self) -> bool {
        self.has_message.is_some() && self.is_global.is_some()
    }
}

#[derive(Resource)]
pub(crate) struct ChatMessageEvents {
    insert_events: HashMap<Entity, ChatMessageRecord>,
}

impl Default for ChatMessageEvents {
    fn default() -> Self {
        Self {
            insert_events: HashMap::new(),
        }
    }
}

impl ChatMessageEvents {
    pub fn recv_inserted_component_events(
        &mut self,
        rdr: &mut EventReader<SessionInsertComponentEvent<ChatMessage>>,
        global_rdr: &mut EventReader<SessionInsertComponentEvent<ChatMessageGlobal>>,
        local_rdr: &mut EventReader<SessionInsertComponentEvent<ChatMessageLocal>>,
    ) -> Vec<(Entity, bool)> {
        for event in rdr.read() {
            if !self.insert_events.contains_key(&event.entity) {
                self.insert_events
                    .insert(event.entity, ChatMessageRecord::new());
            }
            let record = self.insert_events.get_mut(&event.entity).unwrap();
            record.recv_message();
        }

        for event in global_rdr.read() {
            if !self.insert_events.contains_key(&event.entity) {
                self.insert_events
                    .insert(event.entity, ChatMessageRecord::new());
            }
            let record = self.insert_events.get_mut(&event.entity).unwrap();
            record.recv_global();
        }

        for event in local_rdr.read() {
            if !self.insert_events.contains_key(&event.entity) {
                self.insert_events
                    .insert(event.entity, ChatMessageRecord::new());
            }
            let record = self.insert_events.get_mut(&event.entity).unwrap();
            record.recv_local();
        }

        let mut results = Vec::new();
        for (entity, record) in self.insert_events.iter() {
            if record.is_done() {
                results.push((*entity, record.is_global.unwrap()));
            }
        }
        for (entity, _) in results.iter() {
            self.insert_events.remove(entity);
        }
        results
    }
}
