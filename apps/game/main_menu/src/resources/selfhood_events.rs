use std::collections::HashMap;

use bevy_ecs::{entity::Entity, event::EventReader, prelude::Resource};

use game_engine::logging::info;

use game_app_network::session::{
    components::{Selfhood, SelfhoodUser},
    SessionInsertComponentEvent,
};

struct SelfhoodRecord {
    has_selfhood: Option<()>,
    has_user: Option<()>,
}

impl SelfhoodRecord {
    pub fn new() -> Self {
        Self {
            has_selfhood: None,
            has_user: None,
        }
    }

    pub fn recv_selfhood(&mut self) {
        self.has_selfhood = Some(());
    }

    pub fn recv_user(&mut self) {
        self.has_user = Some(());
    }

    pub fn is_done(&self) -> bool {
        self.has_selfhood.is_some() && self.has_user.is_some()
    }
}

#[derive(Resource)]
pub(crate) struct SelfhoodEvents {
    insert_events: HashMap<Entity, SelfhoodRecord>,
}

impl Default for SelfhoodEvents {
    fn default() -> Self {
        Self {
            insert_events: HashMap::new(),
        }
    }
}

impl SelfhoodEvents {
    pub fn recv_inserted_component_events(
        &mut self,
        rdr: &mut EventReader<SessionInsertComponentEvent<Selfhood>>,
        user_rdr: &mut EventReader<SessionInsertComponentEvent<SelfhoodUser>>,
    ) -> Vec<Entity> {
        for event in rdr.read() {
            info!(
                "received Inserted Selfhood from Session Server!  [ {:?} ]",
                &event.entity
            );
            if !self.insert_events.contains_key(&event.entity) {
                self.insert_events
                    .insert(event.entity, SelfhoodRecord::new());
            }
            let record = self.insert_events.get_mut(&event.entity).unwrap();
            record.recv_selfhood();
        }

        for event in user_rdr.read() {
            info!(
                "received Inserted SelfhoodUser from Session Server!  [ {:?} ]",
                &event.entity
            );
            if !self.insert_events.contains_key(&event.entity) {
                self.insert_events
                    .insert(event.entity, SelfhoodRecord::new());
            }
            let record = self.insert_events.get_mut(&event.entity).unwrap();
            record.recv_user();
        }

        let mut results = Vec::new();
        for (entity, record) in self.insert_events.iter() {
            if record.is_done() {
                results.push(*entity);
            }
        }
        for entity in results.iter() {
            self.insert_events.remove(entity);
        }
        results
    }
}
