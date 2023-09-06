use std::{
    collections::{HashMap, HashSet},
    fs,
    path::Path,
};

use bevy_ecs::{
    entity::Entity,
    system::{Commands, ResMut, Resource, SystemState},
    world::World,
};
use bevy_log::info;
use git2::{Cred, Repository, Tree};

use naia_bevy_server::{BigMap, CommandsExt, ReplicationConfig, RoomKey, Server, UserKey};

use vortex_proto::{
    components::{EntryKind, FileSystemChild, FileSystemEntry, FileSystemRootChild},
    messages::ChangelistMessage,
    resources::FileEntryKey,
    FileExtension,
};

use crate::{
    config::GitConfig,
    files::{FileWriter, ShapeType},
    resources::{
        project::Project, project::ProjectKey, ContentEntityData, FileEntryValue, ShapeManager,
        UserManager,
    },
};

#[derive(Resource)]
pub struct GitManager {
    config: Option<GitConfig>,
    projects: BigMap<ProjectKey, Project>,
    // get project key from username, should only be used on initialization
    project_keys: HashMap<String, ProjectKey>,
    content_entity_keys: HashMap<Entity, (ProjectKey, FileEntryKey)>,
    queued_client_modify_files: Vec<(ProjectKey, FileEntryKey)>,
}

impl Default for GitManager {
    fn default() -> Self {
        Self {
            config: None,
            projects: BigMap::new(),
            project_keys: HashMap::new(),
            content_entity_keys: HashMap::new(),
            queued_client_modify_files: Vec::new(),
        }
    }
}

impl GitManager {
    pub fn use_config(&mut self, config: &GitConfig) {
        self.config = Some(config.clone());
    }

    pub fn has_project_key(&mut self, project_owner_name: &str) -> bool {
        self.project_keys.contains_key(project_owner_name)
    }

    pub fn project_key_from_name(&self, project_owner_name: &str) -> Option<ProjectKey> {
        self.project_keys.get(project_owner_name).cloned()
    }

    pub fn project(&self, project_key: &ProjectKey) -> Option<&Project> {
        self.projects.get(project_key)
    }

    pub fn project_mut(&mut self, project_key: &ProjectKey) -> Option<&mut Project> {
        self.projects.get_mut(project_key)
    }

    pub(crate) fn file_entity(
        &self,
        project_key: &ProjectKey,
        file_key: &FileEntryKey,
    ) -> Option<Entity> {
        let project = self.projects.get(project_key).unwrap();
        project.file_entity(file_key)
    }

    pub(crate) fn on_client_create_file(
        &mut self,
        commands: &mut Commands,
        server: &mut Server,
        project_key: &ProjectKey,
        file_name: &str,
        file_entity: Entity,
        parent_file_key: Option<FileEntryKey>,
        file_key: &FileEntryKey,
    ) {
        let project = self.projects.get_mut(project_key).unwrap();
        project.on_client_create_file(
            commands,
            server,
            file_name,
            file_entity,
            parent_file_key,
            file_key,
        );
    }

    pub(crate) fn content_entity_keys(
        &self,
        content_entity: &Entity,
    ) -> Option<(ProjectKey, FileEntryKey)> {
        self.content_entity_keys.get(content_entity).cloned()
    }

    pub(crate) fn on_client_modify_file(
        &mut self,
        commands: &mut Commands,
        server: &mut Server,
        project_key: &ProjectKey,
        file_key: &FileEntryKey,
    ) {
        if file_key.kind() == EntryKind::Directory {
            return;
        }
        let file_entity = self.file_entity(&project_key, &file_key).unwrap();
        let project = self.projects.get_mut(project_key).unwrap();
        project.on_client_modify_file(commands, server, file_key, &file_entity);
    }

    pub(crate) fn queue_client_modify_file(&mut self, content_entity: &Entity) {
        let Some((project_key, file_key)) = self.content_entity_keys(content_entity) else {
            panic!("Could not find content entity key for entity: {:?}", content_entity);
        };
        self.queued_client_modify_files
            .push((project_key, file_key));
    }

    pub(crate) fn process_queued_actions(world: &mut World) {
        let mut system_state: SystemState<(Commands, Server, ResMut<GitManager>)> =
            SystemState::new(world);
        let (mut commands, mut server, mut git_manager) = system_state.get_mut(world);

        for (project_key, file_key) in std::mem::take(&mut git_manager.queued_client_modify_files) {
            git_manager.on_client_modify_file(&mut commands, &mut server, &project_key, &file_key);
        }

        system_state.apply(world);
    }

    pub(crate) fn on_client_insert_content_entity(
        &mut self,
        server: &mut Server,
        project_key: &ProjectKey,
        file_key: &FileEntryKey,
        entity: &Entity,
        shape_type: ShapeType,
    ) {
        self.content_entity_keys
            .insert(*entity, (*project_key, file_key.clone()));

        self.queue_client_modify_file(entity);

        let project = self.projects.get_mut(&project_key).unwrap();
        project.on_insert_content_entity(file_key, entity, shape_type);

        let file_room_key = project.file_room_key(file_key).unwrap();
        server.room_mut(&file_room_key).add_entity(entity);
    }

    pub(crate) fn on_client_remove_content_entity(&mut self, entity: &Entity) {
        self.queue_client_modify_file(entity);

        let Some((project_key, file_key)) = self.content_entity_keys.remove(entity) else {
            panic!("Could not find content entity key for entity: {:?}", entity);
        };

        let project = self.projects.get_mut(&project_key).unwrap();
        project.on_remove_content_entity(&file_key, entity);
    }

    pub(crate) fn user_join_filespace(
        &mut self,
        commands: &mut Commands,
        server: &mut Server,
        shape_manager: &mut ShapeManager,
        user_key: &UserKey,
        project_key: &ProjectKey,
        file_key: &FileEntryKey,
    ) {
        let Some(project) = self.projects.get_mut(project_key) else {
            panic!("Could not find project for user");
        };
        if let Some(new_content_entities) =
            project.user_join_filespace(commands, server, shape_manager, user_key, file_key)
        {
            for entity in new_content_entities {
                self.content_entity_keys
                    .insert(entity, (*project_key, file_key.clone()));
            }
        }
    }

    pub fn create_project(
        &mut self,
        commands: &mut Commands,
        server: &mut Server,
        owner_name: &str,
    ) -> ProjectKey {
        // Create User's Working directory if it doesn't already exist
        let root_dir = "target/users";
        let full_path_str = format!("{}/{}", root_dir, owner_name);
        let path = Path::new(&full_path_str);
        let repo_url = self.config.as_ref().unwrap().repo_url.as_str();
        let access_token = self.config.as_ref().unwrap().access_token.as_str();

        let mut fetch_options = git2::FetchOptions::new();
        fetch_options.remote_callbacks(Self::get_remote_callbacks(access_token));

        let repo = if !path.exists() {
            // Create new directory
            fs::create_dir_all(path).unwrap();

            // Put fetch options into builder
            let mut builder = git2::build::RepoBuilder::new();
            builder.fetch_options(fetch_options);

            // Clone repo
            let repo = builder.clone(repo_url, path).unwrap();

            info!("initialized repo at: `{}`", &full_path_str);

            repo
        } else {
            info!("repo exists at: `{}`", &full_path_str);

            // Open repo
            let repo = Repository::open(path).unwrap();

            {
                let mut remote = repo.find_remote("origin").unwrap();
                remote
                    .fetch(&["main"], Some(&mut fetch_options), None)
                    .unwrap();

                let reference = repo.find_reference("FETCH_HEAD").unwrap();
                let target = reference.peel_to_commit().unwrap();

                // Set up a CheckoutBuilder to force the working directory to match the target
                let mut checkout_builder = git2::build::CheckoutBuilder::new();
                checkout_builder.force();

                // Reset local changes
                repo.reset(
                    target.as_object(),
                    git2::ResetType::Hard,
                    Some(&mut checkout_builder),
                )
                .unwrap();

                info!("pulled repo with new changes");
            }

            repo
        };

        let mut file_entries = HashMap::new();

        {
            let head = repo.head().unwrap();
            let tree = head.peel_to_tree().unwrap();

            fill_file_entries_from_git(&mut file_entries, commands, server, &repo, &tree, "", None);
        }

        // Create new room for user and all their owned entities
        let project_room_key = server.make_room().key();

        let new_project = Project::new(
            project_room_key,
            file_entries,
            repo,
            access_token,
            &full_path_str,
        );

        insert_entry_components_from_list(
            commands,
            server,
            &new_project.master_file_entries,
            &new_project.room_key(),
        );

        let project_key = self.projects.insert(new_project);
        self.project_keys
            .insert(owner_name.to_string(), project_key);

        project_key
    }

    pub fn get_remote_callbacks(access_token: &str) -> git2::RemoteCallbacks {
        let mut remote_callbacks = git2::RemoteCallbacks::new();
        remote_callbacks.credentials(move |_url, _username_from_url, _allowed_types| {
            Cred::userpass_plaintext("token", access_token)
        });

        remote_callbacks
    }

    pub fn commit_changelist_entry(
        &mut self,
        world: &mut World,
        user_key: UserKey,
        message: ChangelistMessage,
    ) {
        let user_manager = world.get_resource::<UserManager>().unwrap();
        let user_perm_data = user_manager.user_perm_data(&user_key).unwrap();
        let username = user_perm_data.username().to_string();
        let email = user_perm_data.email().to_string();

        let user_session_data = user_manager.user_session_data(&user_key).unwrap();
        let project_key = user_session_data.project_key().unwrap();

        let Some(project) = self.projects.get_mut(&project_key) else {
            panic!("Could not find project for user: `{}`", username);
        };

        project.commit_changelist_entry(world, &username, &email, message);
    }

    pub fn rollback_changelist_entry(
        &mut self,
        world: &mut World,
        user_key: UserKey,
        message: ChangelistMessage,
    ) {
        let user_manager = world.get_resource::<UserManager>().unwrap();

        let user_name = user_manager
            .user_perm_data(&user_key)
            .unwrap()
            .username()
            .to_string();

        let user_session_data = user_manager.user_session_data(&user_key).unwrap();
        let Some(user_project_key) = user_session_data.project_key() else {
            panic!("Could not find project key for user: `{}`", user_name);
        };
        let Some(project) = self.projects.get_mut(&user_project_key) else {
            panic!("Could not find project for user: `{}`", user_name);
        };
        let project_room_key = project.room_key();

        if let Some((key, value)) = project.rollback_changelist_entry(world, message) {
            self.spawn_networked_entry_into_world(
                world,
                &user_project_key,
                &project_room_key,
                &key,
                &value,
            )
        }
    }

    fn spawn_networked_entry_into_world(
        &mut self,
        world: &mut World,
        project_key: &ProjectKey,
        project_room_key: &RoomKey,
        entry_key: &FileEntryKey,
        entry_val: &FileEntryValue,
    ) {
        let project = self.projects.get(project_key).unwrap();

        let mut system_state: SystemState<(Commands, Server)> = SystemState::new(world);
        let (mut commands, mut server) = system_state.get_mut(world);

        insert_entry_components(
            &mut commands,
            &mut server,
            project_room_key,
            &project.working_file_entries,
            entry_key,
            entry_val,
        );

        system_state.apply(world);
    }

    pub(crate) fn can_read(&self, project_key: &ProjectKey, key: &FileEntryKey) -> bool {
        let ext = self.working_file_extension(project_key, key);
        return ext.can_io();
    }

    pub(crate) fn can_write(&self, project_key: &ProjectKey, key: &FileEntryKey) -> bool {
        let ext = self.working_file_extension(project_key, key);
        return ext.can_io();
    }

    pub(crate) fn write(
        &self,
        project_key: &ProjectKey,
        key: &FileEntryKey,
        world: &mut World,
        content_entities: &HashMap<Entity, ContentEntityData>,
    ) -> Box<[u8]> {
        let ext = self.working_file_extension(project_key, key);
        return ext.write(world, content_entities);
    }

    pub fn working_file_extension(
        &self,
        project_key: &ProjectKey,
        key: &FileEntryKey,
    ) -> FileExtension {
        self.projects
            .get(project_key)
            .unwrap()
            .working_file_extension(key)
    }

    pub(crate) fn set_changelist_entry_content(
        &mut self,
        commands: &mut Commands,
        server: &mut Server,
        project_key: &ProjectKey,
        key: &FileEntryKey,
        bytes: Box<[u8]>,
    ) {
        let project = self.projects.get_mut(project_key).unwrap();
        project.set_changelist_entry_content(commands, server, key, bytes);
    }

    pub fn spawn_file_tree_entity(commands: &mut Commands, server: &mut Server) -> Entity {
        let entity_id = commands
            .spawn_empty()
            .enable_replication(server)
            .configure_replication(ReplicationConfig::Delegated)
            .id();

        entity_id
    }
}

fn fill_file_entries_from_git(
    file_entries: &mut HashMap<FileEntryKey, FileEntryValue>,
    commands: &mut Commands,
    server: &mut Server,
    repo: &Repository,
    git_tree: &Tree,
    path: &str,
    parent: Option<FileEntryKey>,
) -> HashSet<FileEntryKey> {
    let mut output = HashSet::new();

    for git_entry in git_tree.iter() {
        let name = git_entry.name().unwrap().to_string();

        info!("Git -> Tree: processing Entry `{:?}`", name);

        match git_entry.kind() {
            Some(git2::ObjectType::Tree) => {
                let entry_kind = EntryKind::Directory;
                let id = GitManager::spawn_file_tree_entity(commands, server);

                let file_entry_key = FileEntryKey::new(path, &name, entry_kind);

                let git_children = git_entry.to_object(repo).unwrap().peel_to_tree().unwrap();
                let new_path = file_entry_key.path_for_children();
                let children = fill_file_entries_from_git(
                    file_entries,
                    commands,
                    server,
                    repo,
                    &git_children,
                    &new_path,
                    Some(file_entry_key.clone()),
                );

                let file_entry_value =
                    FileEntryValue::new(id, parent.clone(), Some(children), None);
                file_entries.insert(file_entry_key.clone(), file_entry_value);

                output.insert(file_entry_key.clone());
            }
            Some(git2::ObjectType::Blob) => {
                let entry_kind = EntryKind::File;
                let id = GitManager::spawn_file_tree_entity(commands, server);

                let file_entry_key = FileEntryKey::new(path, &name, entry_kind);
                let file_extension = FileExtension::from_file_name(&name);
                let file_entry_value =
                    FileEntryValue::new(id, parent.clone(), None, Some(file_extension));
                file_entries.insert(file_entry_key.clone(), file_entry_value);

                output.insert(file_entry_key.clone());
            }
            _ => {
                info!("Unknown file type: {:?}", git_entry.kind());
            }
        }
    }

    output
}

fn insert_entry_components_from_list(
    commands: &mut Commands,
    server: &mut Server,
    file_entries: &HashMap<FileEntryKey, FileEntryValue>,
    project_room_key: &RoomKey,
) {
    for (file_entry_key, file_entry_value) in file_entries.iter() {
        info!(
            "Networking: walking tree for Entry `{:?}`",
            file_entry_key.name()
        );

        insert_entry_components(
            commands,
            server,
            project_room_key,
            file_entries,
            file_entry_key,
            file_entry_value,
        );
    }
}

fn insert_entry_components(
    commands: &mut Commands,
    server: &mut Server,
    project_room_key: &RoomKey,
    file_entries: &HashMap<FileEntryKey, FileEntryValue>,
    entry_key: &FileEntryKey,
    entry_val: &FileEntryValue,
) {
    let entry_entity = entry_val.entity();

    // Insert components
    commands
        .entity(entry_entity)
        .insert(FileSystemEntry::new(&entry_key.name(), entry_key.kind()))
        .insert(entry_key.clone());

    // Add entity to room
    server.room_mut(project_room_key).add_entity(&entry_entity);

    // Add parent entity to component
    if let Some(parent_key) = entry_val.parent() {
        let parent_entity = file_entries.get(parent_key).unwrap().entity();

        let mut parent_component = FileSystemChild::new();
        parent_component.parent_id.set(server, &parent_entity);
        commands.entity(entry_entity).insert(parent_component);
    } else {
        commands.entity(entry_entity).insert(FileSystemRootChild);
    }
}
