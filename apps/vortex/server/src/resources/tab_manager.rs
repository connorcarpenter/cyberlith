use std::collections::{HashMap, HashSet, VecDeque};

use bevy_ecs::{
    entity::Entity,
    system::{Commands, Query, ResMut, Resource, SystemState},
    world::{Mut, World},
};
use bevy_log::{info, warn};
use naia_bevy_server::{CommandsExt, Server, UserKey};

use vortex_proto::{resources::FileEntryKey, types::TabId};

use crate::resources::{user_tab_state::TabState, GitManager, UserManager, UserTabState};

#[derive(Resource)]
pub struct TabManager {
    users: HashMap<UserKey, UserTabState>,
    queued_closes: VecDeque<(UserKey, TabId)>,
    waiting_opens: HashMap<(UserKey, Entity), TabId>,
}

impl Default for TabManager {
    fn default() -> Self {
        Self {
            users: HashMap::new(),
            queued_closes: VecDeque::new(),
            waiting_opens: HashMap::new(),
        }
    }
}

impl TabManager {
    pub fn open_tab(
        &mut self,
        commands: &mut Commands,
        server: &mut Server,
        user_manager: &UserManager,
        git_manager: &mut GitManager,
        key_query: &Query<&FileEntryKey>,
        user_key: &UserKey,
        tab_id: &TabId,
        file_entity: &Entity,
    ) {
        let Ok(file_entry_key) = key_query.get(*file_entity) else {
            self.waiting_opens.insert((*user_key, *file_entity), *tab_id);
            return;
        };

        // load from file all Entities in the file of the current tab
        let username = user_manager.user_name(user_key).unwrap();

        if !git_manager.can_read(username, &file_entry_key) {
            warn!("can't read file: `{:?}`", file_entry_key.name());
            return;
        }

        // initialize user tab state if necessary
        if !self.users.contains_key(user_key) {
            self.users.insert(user_key.clone(), UserTabState::default());
        }

        let user_state = self.users.get_mut(user_key).unwrap();

        // create new Room for entities which are in the new tab
        let new_room_key = server.make_room().key();

        let content_entities =
            git_manager.load_content_entities(commands, server, &file_entry_key, username);

        for entity in content_entities.iter() {
            // associate all new Entities with the new Room
            server.room_mut(&new_room_key).add_entity(entity);

            commands
                .entity(*entity)
                // call "pause_replication" on all Entities (they will be resumed when tab is selected)
                .pause_replication(server);
        }

        // insert TabState into collection
        let tab_state = TabState::new(new_room_key, file_entity.clone(), content_entities);
        user_state.state_tabs_insert(tab_id.clone(), tab_state);

        // put user in new room
        server.room_mut(&new_room_key).add_user(user_key);
    }

    pub fn select_tab(
        &mut self,
        commands: &mut Commands,
        server: &mut Server,
        user_key: &UserKey,
        tab_id: &TabId,
    ) {
        let Some(user_state) = self.users.get_mut(user_key) else {
            warn!("select_tab(): user_tab_state has not been initialized!");
            return;
        };
        if !user_state.has_tab_id(tab_id) {
            warn!("User does not have tab {}", tab_id);
            return;
        }

        info!("Select Tab!");

        if let Some(tab_entities) = user_state.current_tab_entities() {
            for entity in tab_entities.iter() {
                // All Entities associated with previous tab must call "pause_replication"
                commands.entity(*entity).pause_replication(server);
            }
        }

        // Switch current Tab
        user_state.set_current_tab(Some(tab_id.clone()));

        // All Entities associated with new tab must call "resume_replication"
        let new_tab_entities = user_state.tab_entities(tab_id).unwrap();
        for entity in new_tab_entities.iter() {
            commands.entity(*entity).resume_replication(server);
        }
    }

    pub fn remove_waiting_open(
        &mut self,
        user_key: &UserKey,
        file_entity: &Entity,
    ) -> Option<TabId> {
        self.waiting_opens.remove(&(*user_key, *file_entity))
    }

    pub fn queue_close_tab(&mut self, user_key: UserKey, tab_id: TabId) {
        self.queued_closes.push_back((user_key, tab_id));
    }

    pub fn process_queued_actions(world: &mut World) {
        // closed tabs
        let closed_states = {
            let mut system_state: SystemState<(Server, ResMut<TabManager>)> =
                SystemState::new(world);
            let (mut server, mut tab_manager) = system_state.get_mut(world);

            Self::process_queued_actions_inner(&mut tab_manager, &mut server)
        };

        if closed_states.is_empty() {
            return;
        }

        // we need to despawn entities associated with tab, so before that,
        // backup the data to the changelist entry
        {
            world.resource_scope(|world, mut git_manager: Mut<GitManager>| {
                let mut output = Vec::new();

                for (user_key, closed_state) in closed_states.iter() {
                    let username = world
                        .get_resource::<UserManager>()
                        .unwrap()
                        .user_name(&user_key)
                        .unwrap()
                        .to_string();
                    let file_entry_key = world
                        .entity(closed_state.get_file_entity())
                        .get::<FileEntryKey>()
                        .unwrap()
                        .clone();
                    if !git_manager.can_write(&username, &file_entry_key) {
                        panic!("can't write file: `{:?}`", file_entry_key.name());
                    }
                    let bytes = git_manager.write(
                        &username,
                        &file_entry_key,
                        world,
                        closed_state.get_content_entities(),
                    );
                    output.push((username, file_entry_key, bytes));
                }

                let mut system_state: SystemState<(Commands, Server)> = SystemState::new(world);
                let (mut commands, mut server) = system_state.get_mut(world);

                for (username, key, bytes) in output {
                    git_manager.set_changelist_entry_content(
                        &mut commands,
                        &mut server,
                        &username,
                        &key,
                        bytes,
                    );
                }
            });
        }

        // actually despawn entities associated with tab
        {
            for (_user_key, closed_state) in closed_states.iter() {
                // despawn content entities
                let entities = closed_state.get_content_entities();
                for entity in entities.iter() {
                    world.entity_mut(*entity).despawn();
                }
            }
        }
    }

    pub fn on_insert_vertex(&mut self, user_key: &UserKey, vertex_entity: &Entity) {
        self.user_tab_state_mut(user_key).current_tab_add_entity(vertex_entity);
    }

    pub fn on_remove_vertex(&mut self, user_key: &UserKey, vertex_entity: &Entity) {
        self.user_tab_state_mut(user_key).current_tab_remove_entity(vertex_entity);
    }

    pub(crate) fn user_current_tab_has_entity(&self, user_key: &UserKey, entity: &Entity) -> bool {
        if let Some(entities) = self.user_tab_state(user_key).current_tab_entities() {
            entities.contains(entity)
        } else {
            false
        }
    }

    pub(crate) fn user_current_tab_file_entity(&self, user_key: &UserKey) -> Entity {
        self.user_tab_state(user_key).current_tab_file_entity().unwrap()
    }

    pub(crate) fn user_current_tab_content_entities(&self, user_key: &UserKey) -> &HashSet<Entity> {
        self.user_tab_state(user_key).current_tab_entities().unwrap()
    }

    fn user_tab_state(&self, user_key: &UserKey) -> &UserTabState {
        let Some(user_state) = self.users.get(user_key) else {
            panic!("user_tab_state has not been initialized!");
        };

        user_state
    }

    fn user_tab_state_mut(&mut self, user_key: &UserKey) -> &mut UserTabState {
        let Some(user_state) = self.users.get_mut(user_key) else {
            panic!("user_tab_state has not been initialized!");
        };

        user_state
    }

    fn process_queued_actions_inner(&mut self, server: &mut Server) -> Vec<(UserKey, TabState)> {
        let mut output = Vec::new();
        while let Some((user_key, tab_id)) = self.queued_closes.pop_front() {
            let closed_state = self.close_tab(server, &user_key, &tab_id);
            output.push((user_key, closed_state));
        }
        output
    }

    fn close_tab(&mut self, server: &mut Server, user_key: &UserKey, tab_id: &TabId) -> TabState {
        let mut remove = false;
        let Some(user_state) = self.users.get_mut(user_key) else {
            panic!("User does not exist!");
        };
        if user_state.get_current_tab() == Some(tab_id.clone()) {
            user_state.set_current_tab(None);
        }

        let Some(tab_state) = user_state.state_tabs_remove(tab_id) else {
            panic!("User does not have tab {}", tab_id);
        };

        if !user_state.has_tabs() {
            remove = true;
        }
        if remove {
            self.users.remove(user_key);
        }

        // remove the Room
        server.room_mut(&tab_state.get_room_key()).destroy();

        tab_state
    }
}
