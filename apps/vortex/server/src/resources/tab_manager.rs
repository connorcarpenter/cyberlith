use std::collections::{HashMap, HashSet, VecDeque};

use bevy_ecs::system::Res;
use bevy_ecs::{
    entity::Entity,
    system::{Commands, Query, ResMut, Resource, SystemState},
    world::{Mut, World},
};
use bevy_log::{info, warn};
use naia_bevy_server::{CommandsExt, Server, UserKey};

use vortex_proto::{resources::FileEntryKey, types::TabId};

use crate::resources::{
    user_tab_state::TabState, workspace::Workspace, GitManager, UserManager, UserTabState,
    VertexManager,
};

#[derive(Resource)]
pub struct TabManager {
    users: HashMap<UserKey, UserTabState>,
    queued_closes: VecDeque<(UserKey, TabId)>,
    queued_opens: VecDeque<(UserKey, TabId, Entity)>,
    queued_selects: VecDeque<(UserKey, TabId)>,
    waiting_opens: HashMap<(UserKey, Entity), TabId>,
    waiting_selects: HashMap<UserKey, TabId>,
}

impl Default for TabManager {
    fn default() -> Self {
        Self {
            users: HashMap::new(),
            queued_closes: VecDeque::new(),
            queued_opens: VecDeque::new(),
            queued_selects: VecDeque::new(),
            waiting_opens: HashMap::new(),
            waiting_selects: HashMap::new(),
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
        vertex_manager: &mut VertexManager,
        key_query: &Query<&FileEntryKey>,
        user_key: &UserKey,
        tab_id: &TabId,
        file_entity: &Entity,
    ) {
        info!("open tab");

        let Ok(file_entry_key) = key_query.get(*file_entity) else {
            self.waiting_opens.insert((*user_key, *file_entity), *tab_id);
            info!("no FileEntryKey for entity: {:?}, queuing open tab", file_entity);
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

        let content_entities = git_manager.load_content_entities(
            commands,
            server,
            vertex_manager,
            username,
            &file_entry_key,
            &new_room_key,
            *tab_id,
            true,
        );

        // insert TabState into collection
        let tab_state = TabState::new(new_room_key, file_entity.clone(), content_entities);
        user_state.insert_tab_state(tab_id.clone(), tab_state);

        // put user in new room
        server.room_mut(&new_room_key).add_user(user_key);

        // if there is a selection waiting, queue it
        if let Some(tab_id) = self.waiting_selects.remove(user_key) {
            self.queued_selects.push_back((*user_key, tab_id));
        }
    }

    pub fn select_tab(
        &mut self,
        commands: &mut Commands,
        server: &mut Server,
        user_key: &UserKey,
        tab_id: &TabId,
    ) {
        let Some(user_state) = self.users.get_mut(user_key) else {
            info!("select_tab(): user_tab_state has not been initialized, queuing for later.");
            self.waiting_selects.insert(*user_key, *tab_id);
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

    pub fn complete_waiting_open(&mut self, user_key: &UserKey, file_entity: &Entity) {
        if let Some(tab_id) = self.waiting_opens.remove(&(*user_key, *file_entity)) {
            self.queued_opens
                .push_back((*user_key, tab_id, *file_entity));
        }
    }

    pub fn queue_close_tab(&mut self, user_key: UserKey, tab_id: TabId) {
        self.queued_closes.push_back((user_key, tab_id));
    }

    pub fn process_queued_actions(world: &mut World) {
        Self::process_queued_opens(world);
        Self::process_queued_closes(world);
        Self::process_queued_selects(world);
    }

    fn process_queued_opens(world: &mut World) {
        let mut system_state: SystemState<(
            Commands,
            Server,
            ResMut<TabManager>,
            Res<UserManager>,
            ResMut<GitManager>,
            ResMut<VertexManager>,
            Query<&FileEntryKey>,
        )> = SystemState::new(world);
        let (
            mut commands,
            mut server,
            mut tab_manager,
            user_manager,
            mut git_manager,
            mut vertex_manager,
            key_query,
        ) = system_state.get_mut(world);

        let opens = tab_manager.take_queued_opens();

        for (user_key, tab_id, file_entity) in opens {
            tab_manager.open_tab(
                &mut commands,
                &mut server,
                &user_manager,
                &mut git_manager,
                &mut vertex_manager,
                &key_query,
                &user_key,
                &tab_id,
                &file_entity,
            );
        }

        system_state.apply(world);
    }

    fn take_queued_opens(&mut self) -> VecDeque<(UserKey, TabId, Entity)> {
        std::mem::take(&mut self.queued_opens)
    }

    fn process_queued_closes(world: &mut World) {
        // closed tabs
        let closed_states = {
            let mut system_state: SystemState<(Server, ResMut<TabManager>)> =
                SystemState::new(world);
            let (mut server, mut tab_manager) = system_state.get_mut(world);

            let output = Self::process_queued_closes_inner(&mut tab_manager, &mut server);

            system_state.apply(world);

            output
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

                system_state.apply(world);
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

    fn process_queued_selects(world: &mut World) {
        let mut system_state: SystemState<(Commands, Server, ResMut<TabManager>)> =
            SystemState::new(world);
        let (mut commands, mut server, mut tab_manager) = system_state.get_mut(world);

        let selects = tab_manager.take_queued_selects();

        for (user_key, tab_id) in selects {
            tab_manager.select_tab(&mut commands, &mut server, &user_key, &tab_id);
        }

        system_state.apply(world);
    }

    fn take_queued_selects(&mut self) -> VecDeque<(UserKey, TabId)> {
        std::mem::take(&mut self.queued_selects)
    }

    pub fn on_insert_vertex(&mut self, user_key: &UserKey, vertex_entity: &Entity) {
        self.user_tab_state_mut(user_key)
            .current_tab_add_entity(vertex_entity);
    }

    pub fn on_remove_vertex(&mut self, user_key: &UserKey, vertex_entity: &Entity) {
        self.user_tab_state_mut(user_key)
            .current_tab_remove_entity(vertex_entity);
    }

    pub(crate) fn user_current_tab_has_entity(&self, user_key: &UserKey, entity: &Entity) -> bool {
        if let Some(entities) = self.user_tab_state(user_key).current_tab_entities() {
            entities.contains(entity)
        } else {
            false
        }
    }

    // pub(crate) fn user_current_tab_print_entities(&self, user_key: &UserKey) {
    //     if let Some(entities) = self.user_tab_state(user_key).current_tab_entities() {
    //         info!("user_current_tab_print_entities: {:?}", entities);
    //     } else {
    //         info!("user_current_tab_print_entities: None");
    //     }
    // }

    pub(crate) fn user_current_tab_file_entity(&self, user_key: &UserKey) -> Entity {
        self.user_tab_state(user_key)
            .current_tab_file_entity()
            .unwrap()
    }

    pub(crate) fn user_current_tab_content_entities(&self, user_key: &UserKey) -> &HashSet<Entity> {
        self.user_tab_state(user_key)
            .current_tab_entities()
            .unwrap()
    }

    pub(crate) fn respawn_tab_content_entities(
        &mut self,
        commands: &mut Commands,
        server: &mut Server,
        workspace: &Workspace,
        vertex_manager: &mut VertexManager,
        user_key: &UserKey,
        file_entity: &Entity,
        file_entry_key: &FileEntryKey,
    ) {
        if let Some(user_tab_state) = self.users.get_mut(user_key) {
            user_tab_state.respawn_tab_content_entities(
                commands,
                server,
                workspace,
                vertex_manager,
                file_entity,
                file_entry_key,
            );
        }
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

    fn process_queued_closes_inner(&mut self, server: &mut Server) -> Vec<(UserKey, TabState)> {
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

        let Some(tab_state) = user_state.remove_tab_state(tab_id) else {
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
