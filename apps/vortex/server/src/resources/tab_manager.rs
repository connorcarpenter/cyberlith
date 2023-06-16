use std::collections::HashMap;

use bevy_ecs::{entity::Entity, system::Resource};
use bevy_ecs::system::Commands;
use bevy_log::info;
use naia_bevy_server::{CommandsExt, RoomKey, Server, UserKey};

use vortex_proto::types::TabId;

use crate::resources::GitManager;

struct TabState {
    room_key: RoomKey,
    file_entity: Entity,
    content_entities: Vec<Entity>,
}

impl TabState {
    fn new(room_key: RoomKey, file_entity: Entity, content_entities: Vec<Entity>) -> Self {
        Self {
            room_key,
            file_entity,
            content_entities,
        }
    }

    fn add_content_entity(&mut self, entity: Entity) {
        self.content_entities.push(entity);
    }

    fn remove_content_entity(&mut self, entity: &Entity) {
        self.content_entities.retain(|e| e != entity);
    }
}

struct UserTabState {
    current_tab: Option<TabId>,
    tabs: HashMap<TabId, TabState>,
}

impl Default for UserTabState {
    fn default() -> Self {
        Self {
            current_tab: None,
            tabs: HashMap::new(),
        }
    }
}

#[derive(Resource)]
pub struct TabManager {
    users: HashMap<UserKey, UserTabState>,
}

impl Default for TabManager {
    fn default() -> Self {
        Self {
            users: HashMap::new()
        }
    }
}

impl TabManager {
    pub fn open_tab(
        &mut self,
        commands: &mut Commands,
        server: &mut Server,
        git_manager: &mut GitManager,
        user_key: &UserKey,
        tab_id: &TabId,
        file_entity: &Entity,
    ) {
        if !self.users.contains_key(user_key) {
            self.users.insert(user_key.clone(), UserTabState::default());
        }

        let user_state = self.users.get_mut(user_key).unwrap();

        // create new Room for entities which are in the new tab
        let new_room_key = server.make_room().key();

        // load from file all Entities in the file of the current tab
        let content_entities = git_manager.load_content_entities(commands, &new_room_key, file_entity, tab_id);

        // insert TabState into collection
        let tab_state = TabState::new(new_room_key, file_entity.clone(), content_entities);
        user_state.tabs.insert(tab_id.clone(), tab_state);
    }

    pub fn close_tab(
        &mut self,
        commands: &mut Commands,
        server: &mut Server,
        git_manager: &mut GitManager,
        user_key: &UserKey,
        tab_id: &TabId,
    ) {
        let mut remove = false;
        let Some(user_state) = self.users.get_mut(user_key) else {
            panic!("User does not exist!");
        };
        if user_state.current_tab == Some(tab_id.clone()) {
            user_state.current_tab = None;
        }

        let Some(tab_state) = user_state.tabs.remove(tab_id) else {
            panic!("User does not have tab {}", tab_id);
        };

        if user_state.tabs.is_empty() {
            remove = true;
        }
        if remove {
            self.users.remove(user_key);
        }

        // despawn all Entities in the Room associated with the TabId
        git_manager.unload_content_entities(commands, &tab_state.file_entity, tab_id, tab_state.content_entities);

        // remove the Room
        server.room_mut(&tab_state.room_key).destroy();
    }

    pub fn select_tab(
        &mut self,
        commands: &mut Commands,
        server: &mut Server,
        user_key: &UserKey,
        tab_id: &TabId,
    ) {
        let Some(user_state) = self.users.get_mut(user_key) else {
            panic!("User does not exist!");
        };
        if !user_state.tabs.contains_key(tab_id) {
            panic!("User does not have tab {}", tab_id);
        }

        info!("Select Tab!");


        if let Some(current_tab_id) = user_state.current_tab {
            let Some(current_tab_state) = user_state.tabs.get_mut(&current_tab_id) else {
                panic!("User does not have tab {}", current_tab_id);
            };
            for entity in current_tab_state.content_entities.iter() {
                // All Entities associated with previous tab must call "pause_replication"
                commands.entity(*entity).pause_replication(server);
            }
        }

        // Switch current Tab
        user_state.current_tab = Some(tab_id.clone());

        // All Entities associated with new tab must call "resume_replication"
        let new_tab_state = user_state.tabs.get_mut(tab_id).unwrap();
        for entity in new_tab_state.content_entities.iter() {
            commands.entity(*entity).resume_replication(server);
        }
    }
}