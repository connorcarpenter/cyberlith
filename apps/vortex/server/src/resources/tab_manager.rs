use std::collections::{HashMap, VecDeque};

use bevy_ecs::{
    entity::Entity,
    system::{Commands, Query, Res, ResMut, Resource, SystemState},
    world::{Mut, World},
};
use bevy_log::{info, warn};

use naia_bevy_server::{CommandsExt, Server, UserKey};

use vortex_proto::{resources::FileEntryKey, types::TabId};

use crate::{
    files::ShapeType,
    resources::{
        project::Project, ContentEntityData, GitManager,
        ShapeManager, ShapeWaitlist, UserManager, UserTabState,
    },
};

#[derive(Resource)]
pub struct TabManager {
    queued_closes: VecDeque<(UserKey, TabId)>,
    queued_opens: VecDeque<(UserKey, TabId, Entity)>,
    queued_selects: VecDeque<(UserKey, TabId)>,
    waiting_opens: HashMap<(UserKey, Entity), TabId>,
    waiting_selects: HashMap<UserKey, TabId>,
}

impl Default for TabManager {
    fn default() -> Self {
        Self {
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
        user_manager: &mut UserManager,
        git_manager: &mut GitManager,
        shape_waitlist: &mut ShapeWaitlist,
        shape_manager: &mut ShapeManager,
        key_q: &Query<&FileEntryKey>,
        user_key: &UserKey,
        tab_id: &TabId,
        file_entity: &Entity,
    ) {
        info!("open tab");

        let Ok(file_entry_key) = key_q.get(*file_entity) else {
            self.waiting_opens.insert((*user_key, *file_entity), *tab_id);
            info!("no FileEntryKey for entity: {:?}, queuing open tab", file_entity);
            return;
        };

        // load from file all Entities in the file of the current tab
        let user_session_data = user_manager.user_session_data(user_key).unwrap();
        let project_key = user_session_data.project_key().unwrap();

        if !git_manager.can_read(&project_key, &file_entry_key) {
            warn!("can't read file: `{:?}`", file_entry_key.name());
            return;
        }

        // insert tab into collection
        user_manager.open_tab(user_key, tab_id.clone(), file_entry_key.clone());

        git_manager.user_join_filespace(
            commands,
            server,
            shape_waitlist,
            shape_manager,
            user_key,
            &project_key,
            &file_entry_key,
            *tab_id,
        );
    }

    pub fn select_tab(
        &mut self,
        user_manager: &mut UserManager,
        user_key: &UserKey,
        tab_id: &TabId,
    ) {
        let Some(user_tab_state) = user_manager.user_tab_state_mut(user_key) else {
            panic!("user does not exist")
        };
        if !user_tab_state.has_tab_id(tab_id) {
            warn!("User does not have tab {}", tab_id);
            return;
        }

        info!("Select Tab!");

        // Switch current Tab
        user_tab_state.set_current_tab(Some(tab_id.clone()));
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
            ResMut<UserManager>,
            ResMut<GitManager>,
            ResMut<ShapeWaitlist>,
            ResMut<ShapeManager>,
            Query<&FileEntryKey>,
        )> = SystemState::new(world);
        let (
            mut commands,
            mut server,
            mut tab_manager,
            mut user_manager,
            mut git_manager,
            mut vertex_waitlist,
            mut shape_manager,
            key_query,
        ) = system_state.get_mut(world);

        let opens = tab_manager.take_queued_opens();

        for (user_key, tab_id, file_entity) in opens {
            tab_manager.open_tab(
                &mut commands,
                &mut server,
                &mut user_manager,
                &mut git_manager,
                &mut vertex_waitlist,
                &mut shape_manager,
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
                    if !git_manager.can_write(user_key, &file_entry_key) {
                        panic!("can't write file: `{:?}`", file_entry_key.name());
                    }
                    let bytes = git_manager.write(
                        user_key,
                        &file_entry_key,
                        world,
                        closed_state.get_content_entities(),
                    );
                    output.push((user_key, file_entry_key, bytes));
                }

                let mut system_state: SystemState<(Commands, Server)> = SystemState::new(world);
                let (mut commands, mut server) = system_state.get_mut(world);

                for (user_key, key, bytes) in output {
                    git_manager.set_changelist_entry_content(
                        &mut commands,
                        &mut server,
                        user_key,
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
                for (entity, _data) in entities.iter() {
                    world.entity_mut(*entity).despawn();
                }
            }
        }
    }

    fn process_queued_selects(world: &mut World) {
        let mut system_state: SystemState<(ResMut<TabManager>, ResMut<UserManager>)> =
            SystemState::new(world);
        let (mut tab_manager, mut user_manager) = system_state.get_mut(world);

        let selects = tab_manager.take_queued_selects();

        for (user_key, tab_id) in selects {
            tab_manager.select_tab(&mut user_manager, &user_key, &tab_id);
        }

        system_state.apply(world);
    }

    fn take_queued_selects(&mut self) -> VecDeque<(UserKey, TabId)> {
        std::mem::take(&mut self.queued_selects)
    }

    pub fn on_insert_content_entity(
        &mut self,
        user_key: &UserKey,
        entity: &Entity,
        shape_type: ShapeType,
    ) {
        self.user_tab_state_mut(user_key)
            .current_tab_add_content_entity(entity, shape_type);
    }

    pub fn on_remove_content_entity(&mut self, user_key: &UserKey, entity: &Entity) {
        self.user_tab_state_mut(user_key)
            .current_tab_remove_content_entity(entity);
    }

    pub(crate) fn user_current_tab_has_entity(&self, user_key: &UserKey, entity: &Entity) -> bool {
        if let Some(entities) = self.user_tab_state(user_key).current_tab_entities() {
            entities.contains_key(entity)
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

    pub(crate) fn user_current_tab_content_entities(
        &self,
        user_key: &UserKey,
    ) -> &HashMap<Entity, ContentEntityData> {
        self.user_tab_state(user_key)
            .current_tab_entities()
            .unwrap()
    }

    fn process_queued_closes_inner(&mut self, server: &mut Server, user_manager: &mut UserManager) -> Vec<(UserKey, FileSpace)> {
        let mut output = Vec::new();
        while let Some((user_key, tab_id)) = self.queued_closes.pop_front() {
            let closed_state = user_manager.close_tab(server, &user_key, &tab_id);
            output.push((user_key, closed_state));
        }
        output
    }
}
