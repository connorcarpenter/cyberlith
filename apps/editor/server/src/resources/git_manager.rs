use std::collections::{HashMap, HashSet};

use bevy_ecs::{
    entity::Entity,
    system::{Commands, Query, Res, ResMut, Resource, SystemState},
    world::{Mut, World},
};

use asset_serde::json::AssetMeta;
use logging::info;
use naia_bevy_server::{BigMap, CommandsExt, ReplicationConfig, RoomKey, Server, UserKey};

use editor_proto::{
    components::{
        BackgroundSkinColor, EntryKind, FaceColor, FileExtension, FileSystemChild, FileSystemEntry,
        FileSystemRootChild, IconFace,
    },
    messages::ChangelistMessage,
    resources::FileKey,
};
use git::{repo_init, ObjectType, Repository, Tree};

use crate::resources::{
    project::Project, project::ProjectKey, ContentEntityData, FileEntryValue, PaletteManager,
    RollbackResult, ShapeManager, SkinManager, UserManager,
};

#[derive(Resource)]
pub struct GitManager {
    projects: BigMap<ProjectKey, Project>,
    // get project key from username, should only be used on initialization
    project_keys: HashMap<String, ProjectKey>,
    content_entity_keys: HashMap<Entity, (ProjectKey, FileKey)>,
    queued_client_modify_files: Vec<(ProjectKey, FileKey)>,
    queued_client_open_dependency: Vec<(UserKey, ProjectKey, FileKey)>,
}

impl Default for GitManager {
    fn default() -> Self {
        Self {
            projects: BigMap::new(),
            project_keys: HashMap::new(),
            content_entity_keys: HashMap::new(),
            queued_client_modify_files: Vec::new(),
            queued_client_open_dependency: Vec::new(),
        }
    }
}

impl GitManager {
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
        file_key: &FileKey,
    ) -> Option<Entity> {
        let project = self.projects.get(project_key).unwrap();
        project.file_entity(file_key)
    }

    pub(crate) fn user_join_filespace(
        &mut self,
        world: &mut World,
        user_key: &UserKey,
        project_key: &ProjectKey,
        file_key: &FileKey,
    ) {
        let mut new_content_entities_opt = None;
        let mut file_has_dependencies = false;

        info!("user joining file: {:?}", file_key);

        {
            // user join filespace
            let project = self.projects.get_mut(project_key).unwrap();
            if let Some(new_content_entities) =
                project.user_join_filespace(world, user_key, file_key)
            {
                self.register_content_entities(world, project_key, file_key, &new_content_entities);
                new_content_entities_opt = Some(new_content_entities);
            }

            // user join dependency filespaces
            // get the newly created/associated dependency keys after read above (in user_join_filespace())
            let project = self.projects.get_mut(project_key).unwrap();
            if let Some(dependency_file_keys) = project.dependency_file_keys(file_key) {
                if !dependency_file_keys.is_empty() {
                    file_has_dependencies = true;
                }
                for dependency_key in dependency_file_keys {
                    self.user_join_filespace(world, user_key, project_key, &dependency_key);
                }
            }
        }

        {
            // process content entities that depend on dependencies
            if file_has_dependencies {
                if let Some(content_entities) = new_content_entities_opt {
                    self.process_content_entities_with_dependencies(
                        world,
                        project_key,
                        file_key,
                        content_entities,
                    );
                }
            }
        }
    }

    fn process_content_entities_with_dependencies(
        &mut self,
        world: &mut World,
        project_key: &ProjectKey,
        file_key: &FileKey,
        content_entities: HashMap<Entity, ContentEntityData>,
    ) {
        info!("processing content entities with dependencies");
        let mut system_state: SystemState<(
            Server,
            Res<ShapeManager>,
            Res<PaletteManager>,
            ResMut<SkinManager>,
            Query<&mut BackgroundSkinColor>,
            Query<&mut FaceColor>,
            Query<&mut IconFace>,
        )> = SystemState::new(world);
        let (
            server,
            shape_manager,
            palette_manager,
            mut skin_manager,
            mut bckg_color_q,
            mut face_color_q,
            mut icon_face_q,
        ) = system_state.get_mut(world);

        for (entity, data) in content_entities {
            match data {
                ContentEntityData::BackgroundColor(file_data_opt) => {
                    let Some(palette_index) = file_data_opt else {
                        panic!("Could not find file data for entity: {:?}", entity);
                    };

                    let palette_file_entity = self
                        .file_find_dependency(project_key, file_key, FileExtension::Palette)
                        .unwrap();

                    // find palette_color_entity from palette_index
                    let palette_color_entity = palette_manager
                        .color_entity_from_index(&palette_file_entity, palette_index as usize)
                        .unwrap();

                    // set face_3d_entity and palette_color_entity into FaceColor component
                    let Ok(mut bckg_color) = bckg_color_q.get_mut(entity) else {
                        panic!(
                            "Could not find background skin color for entity: {:?}",
                            entity
                        );
                    };
                    bckg_color
                        .palette_color_entity
                        .set(&server, &palette_color_entity);
                    info!("setting palette color entity for new background color");
                }
                ContentEntityData::FaceColor(file_data_opt) => {
                    let Some((face_index, palette_index)) = file_data_opt else {
                        panic!("Could not find file data for entity: {:?}", entity);
                    };

                    let palette_file_entity = self
                        .file_find_dependency(project_key, file_key, FileExtension::Palette)
                        .unwrap();

                    let face_entity;

                    let file_ext = self.file_extension(project_key, file_key);
                    match file_ext {
                        FileExtension::Skin => {
                            let mesh_file_entity = self
                                .file_find_dependency(project_key, file_key, FileExtension::Mesh)
                                .unwrap();

                            // find face_3d_entity from face_index
                            face_entity = shape_manager
                                .face_entity_from_index(&mesh_file_entity, face_index as usize)
                                .unwrap();
                        }
                        _ => panic!("invalid"),
                    }

                    // find palette_color_entity from palette_index
                    let palette_color_entity = palette_manager
                        .color_entity_from_index(&palette_file_entity, palette_index as usize)
                        .unwrap();

                    // set face_entity and palette_color_entity into FaceColor component
                    let Ok(mut face_color) = face_color_q.get_mut(entity) else {
                        panic!("Could not find face color for entity: {:?}", entity);
                    };
                    face_color.face_entity.set(&server, &face_entity);
                    face_color
                        .palette_color_entity
                        .set(&server, &palette_color_entity);

                    // register with skin manager
                    skin_manager.on_create_face_color(&face_entity, &entity);
                }
                ContentEntityData::IconFace(file_data_opt) => {
                    let Some(palette_index) = file_data_opt else {
                        panic!("Could not find file data for entity: {:?}", entity);
                    };

                    let palette_file_entity = self
                        .file_find_dependency(project_key, file_key, FileExtension::Palette)
                        .unwrap();

                    // find palette_color_entity from palette_index
                    let palette_color_entity = palette_manager
                        .color_entity_from_index(&palette_file_entity, palette_index as usize)
                        .unwrap();

                    // set palette_color_entity into IconFace component
                    let Ok(mut icon_face) = icon_face_q.get_mut(entity) else {
                        panic!("Could not find face for entity: {:?}", entity);
                    };
                    icon_face
                        .palette_color_entity
                        .set(&server, &palette_color_entity);
                }
                _ => {}
            }
        }

        system_state.apply(world);
    }

    pub fn file_find_dependency(
        &self,
        project_key: &ProjectKey,
        file_key: &FileKey,
        file_extension: FileExtension,
    ) -> Option<Entity> {
        let project = self.projects.get(project_key).unwrap();
        project.file_find_dependency(file_key, file_extension)
    }

    pub(crate) fn queue_client_open_dependency(
        &mut self,
        user_key: &UserKey,
        project_key: &ProjectKey,
        dependency_key: &FileKey,
    ) {
        self.queued_client_open_dependency
            .push((*user_key, *project_key, dependency_key.clone()));
    }

    fn user_leave_filespace(
        &mut self,
        server: &mut Server,
        project_key: &ProjectKey,
        file_key: &FileKey,
        user_key: &UserKey,
        output: &mut Vec<(
            ProjectKey,
            FileKey,
            Option<HashMap<Entity, ContentEntityData>>,
        )>,
    ) {
        // user leave filespace
        let project = self.projects.get_mut(project_key).unwrap();
        let dependency_file_keys_opt = project.dependency_file_keys(&file_key);
        let content_entities_opt = project.user_leave_filespace(server, user_key, file_key);
        output.push((*project_key, file_key.clone(), content_entities_opt));

        // user leave dependency filespaces
        if let Some(dependency_file_keys) = dependency_file_keys_opt {
            for dependency_key in dependency_file_keys {
                self.user_leave_filespace(server, project_key, &dependency_key, user_key, output);
            }
        }
    }

    pub(crate) fn on_client_close_tab(
        &mut self,
        server: &mut Server,
        project_key: &ProjectKey,
        file_key: &FileKey,
        user_key: &UserKey,
    ) -> Vec<(
        ProjectKey,
        FileKey,
        Option<HashMap<Entity, ContentEntityData>>,
    )> {
        let mut output: Vec<(
            ProjectKey,
            FileKey,
            Option<HashMap<Entity, ContentEntityData>>,
        )> = Vec::new();

        self.user_leave_filespace(server, project_key, file_key, user_key, &mut output);

        output
    }

    pub(crate) fn on_client_create_file(
        &mut self,
        commands: &mut Commands,
        server: &mut Server,
        project_key: &ProjectKey,
        file_name: &str,
        file_entity: Entity,
        parent_file_key: Option<FileKey>,
        file_key: &FileKey,
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

    pub(crate) fn register_content_entities(
        &mut self,
        world: &mut World,
        project_key: &ProjectKey,
        file_key: &FileKey,
        content_entities: &HashMap<Entity, ContentEntityData>,
    ) {
        let mut system_state: SystemState<Server> = SystemState::new(world);
        let mut server = system_state.get_mut(world);

        for (entity, content_entity_data) in content_entities {
            self.on_insert_content_entity(
                &mut server,
                project_key,
                file_key,
                entity,
                content_entity_data,
            );
        }
    }

    fn register_content_entity(
        &mut self,
        project_key: &ProjectKey,
        file_key: &FileKey,
        content_entity: &Entity,
    ) {
        self.content_entity_keys
            .insert(*content_entity, (*project_key, file_key.clone()));
    }

    pub(crate) fn deregister_content_entities(
        &mut self,
        world: &mut World,
        content_entities: &HashMap<Entity, ContentEntityData>,
    ) {
        let mut system_state: SystemState<Server> = SystemState::new(world);
        let mut server = system_state.get_mut(world);

        for (entity, _) in content_entities.iter() {
            self.on_remove_content_entity(&mut server, entity);
        }
    }

    fn deregister_content_entity(
        &mut self,
        content_entity: &Entity,
    ) -> Option<(ProjectKey, FileKey)> {
        self.content_entity_keys.remove(content_entity)
    }

    pub(crate) fn content_entity_keys(
        &self,
        content_entity: &Entity,
    ) -> Option<(ProjectKey, FileKey)> {
        self.content_entity_keys.get(content_entity).cloned()
    }

    pub(crate) fn on_client_modify_file(
        &mut self,
        commands: &mut Commands,
        server: &mut Server,
        project_key: &ProjectKey,
        file_key: &FileKey,
    ) {
        let project = self.projects.get_mut(project_key).unwrap();
        if file_key.kind() == EntryKind::Directory {
            project.on_client_modify_dir(commands, server, file_key);
        } else {
            project.on_client_modify_file(commands, server, file_key);
        }
    }

    pub(crate) fn queue_client_modify_file(&mut self, content_entity: &Entity) {
        let Some((project_key, file_key)) = self.content_entity_keys(content_entity) else {
            panic!(
                "Could not find content entity key for entity: {:?}",
                content_entity
            );
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

        world.resource_scope(|world, mut git_manager: Mut<GitManager>| {
            for (user_key, project_key, dependency_key) in
                std::mem::take(&mut git_manager.queued_client_open_dependency)
            {
                git_manager.user_join_filespace(world, &user_key, &project_key, &dependency_key);
            }
        });
    }

    pub(crate) fn on_insert_content_entity(
        &mut self,
        server: &mut Server,
        project_key: &ProjectKey,
        file_key: &FileKey,
        entity: &Entity,
        content_data: &ContentEntityData,
    ) {
        self.register_content_entity(project_key, file_key, entity);

        let project = self.projects.get_mut(&project_key).unwrap();
        project.on_insert_content_entity(file_key, entity, content_data);

        let file_room_key = project.file_room_key(file_key).unwrap();
        server.room_mut(&file_room_key).add_entity(entity);
    }

    pub(crate) fn on_remove_content_entity(&mut self, server: &mut Server, entity: &Entity) {
        let Some((project_key, file_key)) = self.deregister_content_entity(entity) else {
            panic!("Could not find content entity key for entity: {:?}", entity);
        };

        let project = self.projects.get_mut(&project_key).unwrap();
        project.on_remove_content_entity(&file_key, entity);

        // at this point, the room may have already despawned
        if let Some(file_room_key) = project.file_room_key(&file_key) {
            let mut room_mut = server.room_mut(&file_room_key);
            if room_mut.has_entity(entity) {
                room_mut.remove_entity(entity);
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
        let repo_name = "cyberlith_assets";
        let root_dir = "users";
        let target_path_str = format!("{}/{}", root_dir, owner_name);

        let repo = repo_init(repo_name, &target_path_str);

        let mut file_entries = HashMap::new();

        {
            let head = repo.head().unwrap();
            let tree = head.peel_to_tree().unwrap();

            fill_file_entries_from_git(
                &mut file_entries,
                commands,
                server,
                &repo,
                &tree,
                "",
                None,
                &target_path_str,
            );
        }

        // Create new room for user and all their owned entities
        let project_room_key = server.make_room().key();

        let new_project = Project::new(project_room_key, file_entries, repo, &target_path_str);

        insert_entry_components_from_list(
            commands,
            server,
            &new_project.master_file_entries(),
            &new_project.room_key(),
        );

        let project_key = self.projects.insert(new_project);
        self.project_keys
            .insert(owner_name.to_string(), project_key);

        project_key
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

        match project.rollback_changelist_entry(world, message) {
            RollbackResult::Created => {}
            RollbackResult::Modified(file_key, content_entities_opt) => {
                if let Some((old_content_entities, new_content_entities)) = content_entities_opt {
                    self.deregister_content_entities(world, &old_content_entities);
                    self.register_content_entities(
                        world,
                        &user_project_key,
                        &file_key,
                        &new_content_entities,
                    );
                }
            }
            RollbackResult::Deleted(key, value) => self.spawn_networked_entry_into_world(
                world,
                &user_project_key,
                &project_room_key,
                &key,
                &value,
            ),
        }
    }

    fn spawn_networked_entry_into_world(
        &mut self,
        world: &mut World,
        project_key: &ProjectKey,
        project_room_key: &RoomKey,
        entry_key: &FileKey,
        entry_val: &FileEntryValue,
    ) {
        let project = self.projects.get(project_key).unwrap();

        let mut system_state: SystemState<(Commands, Server)> = SystemState::new(world);
        let (mut commands, mut server) = system_state.get_mut(world);

        insert_entry_components(
            &mut commands,
            &mut server,
            project_room_key,
            &project.working_file_entries(),
            entry_key,
            entry_val,
        );

        system_state.apply(world);
    }

    pub(crate) fn can_read(&self, project_key: &ProjectKey, key: &FileKey) -> bool {
        let ext = self.file_extension(project_key, key);
        return ext.can_io();
    }

    pub(crate) fn can_write(&self, project_key: &ProjectKey, key: &FileKey) -> bool {
        let ext = self.file_extension(project_key, key);
        return ext.can_io();
    }

    pub(crate) fn write(
        &self,
        project_key: &ProjectKey,
        file_key: &FileKey,
        world: &mut World,
        content_entities: &HashMap<Entity, ContentEntityData>,
    ) -> Box<[u8]> {
        let project = self.projects.get(project_key).unwrap();
        let asset_id = project.asset_id(file_key).unwrap();
        project.write(world, file_key, content_entities, &asset_id)
    }

    pub fn file_extension(&self, project_key: &ProjectKey, key: &FileKey) -> FileExtension {
        self.projects
            .get(project_key)
            .unwrap()
            .file_extension(key)
            .unwrap()
    }

    pub(crate) fn set_changelist_entry_content(
        &mut self,
        commands: &mut Commands,
        server: &mut Server,
        project_key: &ProjectKey,
        key: &FileKey,
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
    file_entries: &mut HashMap<FileKey, FileEntryValue>,
    commands: &mut Commands,
    server: &mut Server,
    repo: &Repository,
    git_tree: &Tree,
    path: &str,
    parent: Option<FileKey>,
    full_path_str: &str,
) -> HashSet<FileKey> {
    let mut output = HashSet::new();

    for git_entry in git_tree.iter() {
        let mut name = git_entry.name().unwrap().to_string();
        if name.ends_with(".json") {
            name = name.trim_end_matches(".json").to_string();
        }
        info!("Git -> Tree: processing Entry `{:?}`", name);

        match git_entry.kind() {
            Some(ObjectType::Tree) => {
                let entry_kind = EntryKind::Directory;
                let id = GitManager::spawn_file_tree_entity(commands, server);

                let file_key = FileKey::new(path, &name, entry_kind);

                let git_children = git_entry.to_object(repo).unwrap().peel_to_tree().unwrap();
                let new_path = file_key.full_path();
                let children = fill_file_entries_from_git(
                    file_entries,
                    commands,
                    server,
                    repo,
                    &git_children,
                    &new_path,
                    Some(file_key.clone()),
                    full_path_str,
                );

                let file_entry_value =
                    FileEntryValue::new(id, None, None, parent.clone(), Some(children));
                file_entries.insert(file_key.clone(), file_entry_value);

                output.insert(file_key.clone());
            }
            Some(ObjectType::Blob) => {
                let entry_kind = EntryKind::File;
                let id = GitManager::spawn_file_tree_entity(commands, server);

                let file_key = FileKey::new(path, &name, entry_kind);

                let (asset_id, file_extension) = {
                    let bytes = Project::read_from_file(full_path_str, &file_key);
                    let Ok((asset_meta, file_extension)) = AssetMeta::read_from_file(&bytes) else {
                        panic!("Could not read AssetMeta from file: {:?}", &file_key);
                    };
                    let file_extension = FileExtension::from(file_extension.as_str());
                    (asset_meta.asset_id(), file_extension)
                };

                let file_entry_value = FileEntryValue::new(
                    id,
                    Some(asset_id),
                    Some(file_extension),
                    parent.clone(),
                    None,
                );
                file_entries.insert(file_key.clone(), file_entry_value);

                output.insert(file_key.clone());
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
    file_entries: &HashMap<FileKey, FileEntryValue>,
    project_room_key: &RoomKey,
) {
    for (file_key, file_entry_value) in file_entries.iter() {
        info!("Networking: walking tree for Entry `{:?}`", file_key.name());

        insert_entry_components(
            commands,
            server,
            project_room_key,
            file_entries,
            file_key,
            file_entry_value,
        );
    }
}

fn insert_entry_components(
    commands: &mut Commands,
    server: &mut Server,
    project_room_key: &RoomKey,
    file_entries: &HashMap<FileKey, FileEntryValue>,
    entry_key: &FileKey,
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
