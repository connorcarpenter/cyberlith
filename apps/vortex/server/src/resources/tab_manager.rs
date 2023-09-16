use std::collections::{HashMap, VecDeque};

use bevy_ecs::{
    entity::Entity,
    system::{Commands, Query, ResMut, Resource, SystemState},
    world::{Mut, World},
};
use bevy_log::{info, warn};

use naia_bevy_server::{Server, UserKey};

use vortex_proto::{resources::FileKey, types::TabId};

use crate::{
    files::despawn_file_content_entities,
    resources::{project::ProjectKey, ContentEntityData, GitManager, ShapeManager, UserManager},
};

#[derive(Resource)]
pub struct TabManager {
    queued_closes: VecDeque<(UserKey, TabId)>,
    queued_opens: VecDeque<(UserKey, TabId, Entity)>,
    waiting_opens: HashMap<(UserKey, Entity), TabId>,
}

impl Default for TabManager {
    fn default() -> Self {
        Self {
            queued_closes: VecDeque::new(),
            queued_opens: VecDeque::new(),
            waiting_opens: HashMap::new(),
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
        shape_manager: &mut ShapeManager,
        key_q: &Query<&FileKey>,
        user_key: &UserKey,
        tab_id: &TabId,
        file_entity: &Entity,
    ) {
        info!("open tab");

        let Ok(file_key) = key_q.get(*file_entity) else {
            self.waiting_opens.insert((*user_key, *file_entity), *tab_id);
            info!("no FileEntryKey for entity: {:?}, queuing open tab", file_entity);
            return;
        };

        // load from file all Entities in the file of the current tab
        let user_session_data = user_manager.user_session_data(user_key).unwrap();
        let project_key = user_session_data.project_key().unwrap();

        if !git_manager.can_read(&project_key, &file_key) {
            warn!("can't read file: `{:?}`", file_key.name());
            return;
        }

        // insert tab into collection
        user_manager.open_tab(
            commands,
            server,
            git_manager,
            shape_manager,
            user_key,
            tab_id.clone(),
            &project_key,
            file_key,
        );
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
    }

    fn process_queued_opens(world: &mut World) {
        let mut system_state: SystemState<(
            Commands,
            Server,
            ResMut<TabManager>,
            ResMut<UserManager>,
            ResMut<GitManager>,
            ResMut<ShapeManager>,
            Query<&FileKey>,
        )> = SystemState::new(world);
        let (
            mut commands,
            mut server,
            mut tab_manager,
            mut user_manager,
            mut git_manager,
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
            let mut system_state: SystemState<(
                Server,
                ResMut<TabManager>,
                ResMut<UserManager>,
                ResMut<GitManager>,
            )> = SystemState::new(world);
            let (mut server, mut tab_manager, mut user_manager, mut git_manager) =
                system_state.get_mut(world);

            let output = tab_manager.process_queued_closes_inner(
                &mut server,
                &mut user_manager,
                &mut git_manager,
            );

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

                for (project_key, file_key, content_entities_opt) in closed_states.iter() {
                    if git_manager.file_entity(project_key, file_key).is_none() {
                        // file was deleted, continue
                        continue;
                    }
                    if !git_manager.can_write(project_key, file_key) {
                        panic!("can't write file: `{:?}`", file_key.name());
                    }
                    let bytes = git_manager.write(project_key, file_key, world, content_entities_opt);
                    output.push((project_key, file_key, bytes));
                }

                let mut system_state: SystemState<(Commands, Server)> = SystemState::new(world);
                let (mut commands, mut server) = system_state.get_mut(world);

                for (project_key, key, bytes) in output {
                    git_manager.set_changelist_entry_content(
                        &mut commands,
                        &mut server,
                        project_key,
                        &key,
                        bytes,
                    );
                }

                system_state.apply(world);
            });
        }

        // actually despawn entities associated with tab
        {
            let mut system_state: SystemState<(
                Commands,
                Server,
                ResMut<ShapeManager>,
                ResMut<GitManager>,
            )> = SystemState::new(world);
            let (mut commands, mut server, mut shape_manager, mut git_manager) =
                system_state.get_mut(world);

            for (project_key, file_key, content_entities_opt) in closed_states.iter() {
                if let Some(content_entities) = content_entities_opt {

                    let project = git_manager.project_mut(project_key).unwrap();

                    // handle despawns
                    despawn_file_content_entities(
                        &mut commands,
                        &mut server,
                        &mut shape_manager,
                        project,
                        file_key,
                        content_entities,
                    );

                    // deregister
                    git_manager.deregister_content_entities(&mut server, content_entities);
                }
            }

            system_state.apply(world);
        }
    }

    fn process_queued_closes_inner(
        &mut self,
        server: &mut Server,
        user_manager: &mut UserManager,
        git_manager: &mut GitManager,
    ) -> Vec<(ProjectKey, FileKey, Option<HashMap<Entity, ContentEntityData>>)> {
        let mut output = Vec::new();
        while let Some((user_key, tab_id)) = self.queued_closes.pop_front() {
            let (project_key, file_key, content_entities) =
                user_manager.close_tab(server, git_manager, &user_key, &tab_id);
            output.push((project_key, file_key, content_entities));
        }
        output
    }
}
