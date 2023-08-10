use std::collections::HashSet;

use bevy_ecs::{
    prelude::{Commands, Entity, Query, Resource, World},
    system::{Res, ResMut, SystemState},
};
use bevy_log::{info, warn};

use naia_bevy_client::{Client, CommandsExt, EntityAuthStatus, ReplicationConfig};

use math::Vec3;
use render_api::{
    base::{CpuMaterial, CpuMesh},
    components::Visibility,
    Assets,
};

use vortex_proto::{
    components::{
        FileType, FileTypeValue,
        ChangelistEntry, Edge3d, EntryKind, FileSystemChild, FileSystemEntry, FileSystemRootChild,
        OwnedByTab, Vertex3d,
    },
    types::TabId,
    FileExtension,
};

use crate::app::{
    components::{
        file_system::{ChangelistUiState, FileSystemParent, FileSystemUiState},
        Edge3dLocal, Vertex2d, VertexEntry, VertexTypeData,
    },
    resources::{
        camera_manager::CameraManager,
        canvas::Canvas,
        file_tree::FileTree,
        global::Global,
        tab_manager::TabManager,
        vertex_manager::{CanvasShape, VertexManager},
    },
    systems::file_post_process,
};

#[derive(Clone)]
pub enum Action {
    // A list of File Row entities to select
    SelectEntries(Vec<Entity>),
    // The directory entity to add the new Entry to, the name of the new Entry, it's Kind, an older Entity it was associated with if necessary, and a list of child Entries to create
    NewEntry(
        Option<Entity>,
        String,
        EntryKind,
        Option<Entity>,
        Option<Vec<FileTree>>,
    ),
    // The File Row entity to delete, and a list of entities to select after deleted
    DeleteEntry(Entity, Option<Vec<Entity>>),
    // The File Row entity to rename, and the new name
    RenameEntry(Entity, String),
    // The 2D vertex entity to deselect (or None for deselect)
    SelectVertex(Option<(Entity, CanvasShape)>),
    // Create Vertex (Parent 2d vertex Entity, Position, older vertex 2d entity & 3d entity it was associated with)
    CreateVertex(VertexTypeData, Vec3, Option<(Entity, Entity)>),
    // Delete Vertex (2d vertex entity, optional vertex 2d entity to select after delete)
    DeleteVertex(Entity, Option<(Entity, CanvasShape)>),
    // Move Vertex (2d vertex Entity, Old Position, New Position)
    MoveVertex(Entity, Vec3, Vec3),
}

impl Action {
    pub(crate) fn migrate_file_entities(&mut self, old_entity: Entity, new_entity: Entity) {
        match self {
            Action::SelectEntries(entities) => {
                for entity in entities {
                    if *entity == old_entity {
                        *entity = new_entity;
                    }
                }
            }
            Action::NewEntry(entity_opt, _, _, entity_opt_2, _) => {
                if let Some(entity) = entity_opt {
                    if *entity == old_entity {
                        *entity = new_entity;
                    }
                }
                if let Some(entity) = entity_opt_2 {
                    if *entity == old_entity {
                        *entity = new_entity;
                    }
                }
            }
            Action::DeleteEntry(entity, entities_opt) => {
                if *entity == old_entity {
                    *entity = new_entity;
                }
                if let Some(entities) = entities_opt {
                    for entity in entities {
                        if *entity == old_entity {
                            *entity = new_entity;
                        }
                    }
                }
            }
            Action::RenameEntry(entity, _) => {
                if *entity == old_entity {
                    *entity = new_entity;
                }
            }
            _ => {}
        }
    }

    pub(crate) fn migrate_vertex_entities(
        &mut self,
        old_2d_entity: Entity,
        new_2d_entity: Entity,
        old_3d_entity: Entity,
        new_3d_entity: Entity,
    ) {
        match self {
            Action::SelectVertex(entity_opt) => {
                if let Some((entity, _)) = entity_opt {
                    if *entity == old_2d_entity {
                        *entity = new_2d_entity;
                    }
                }
            }
            Action::CreateVertex(vertex_type_data, _, entity_opt) => {
                vertex_type_data.migrate_vertex_entities(
                    old_2d_entity,
                    new_2d_entity,
                    old_3d_entity,
                    new_3d_entity,
                );

                if let Some((other_2d_entity, other_3d_entity)) = entity_opt {
                    if *other_2d_entity == old_2d_entity {
                        *other_2d_entity = new_2d_entity;
                    }
                    if *other_3d_entity == old_3d_entity {
                        *other_3d_entity = new_3d_entity;
                    }
                }
            }
            Action::DeleteVertex(entity, entity_opt) => {
                if *entity == old_2d_entity {
                    *entity = new_2d_entity;
                }
                if let Some((other_entity, _)) = entity_opt {
                    if *other_entity == old_2d_entity {
                        *other_entity = new_2d_entity;
                    }
                }
            }
            Action::MoveVertex(entity, _, _) => {
                if *entity == old_2d_entity {
                    *entity = new_2d_entity;
                }
            }
            _ => {}
        }
    }
}

#[derive(Resource)]
pub struct ActionStack {
    buffered_actions: Vec<Action>,
    undo_actions: Vec<Action>,
    redo_actions: Vec<Action>,
    undo_enabled: bool,
    redo_enabled: bool,
    buffered_check: bool,
}

impl Default for ActionStack {
    fn default() -> Self {
        Self {
            buffered_actions: Vec::new(),
            undo_actions: Vec::new(),
            redo_actions: Vec::new(),
            undo_enabled: true,
            redo_enabled: true,
            buffered_check: false,
        }
    }
}

impl ActionStack {
    pub fn buffer_action(&mut self, action: Action) {
        self.buffered_actions.push(action);
    }

    pub fn has_undo(&self) -> bool {
        !self.undo_actions.is_empty() && self.undo_enabled
    }

    pub fn has_redo(&self) -> bool {
        !self.redo_actions.is_empty() && self.redo_enabled
    }

    pub fn undo_action(&mut self, world: &mut World) {
        if !self.undo_enabled {
            panic!("Undo is disabled!");
        }
        let Some(action) = self.undo_actions.pop() else {
            panic!("No executed actions to undo!");
        };

        let reversed_action = self.execute_action(world, action);

        self.redo_actions.push(reversed_action);

        self.enable_top(world);
    }

    pub fn redo_action(&mut self, world: &mut World) {
        if !self.redo_enabled {
            panic!("Redo is disabled!");
        }
        let Some(action) = self.redo_actions.pop() else {
            panic!("No undone actions to redo!");
        };

        let reversed_action = self.execute_action(world, action);

        self.undo_actions.push(reversed_action);

        self.enable_top(world);
    }

    pub fn execute_actions(&mut self, world: &mut World) {
        if self.buffered_check {
            self.enable_top(world);
            self.buffered_check = false;
        }
        if self.buffered_actions.is_empty() {
            return;
        }
        let drained_actions: Vec<Action> = self.buffered_actions.drain(..).collect();
        for action in drained_actions {
            let reversed_action = self.execute_action(world, action);
            self.undo_actions.push(reversed_action);
        }
        self.redo_actions.clear();

        self.enable_top(world);
    }

    fn execute_action(&mut self, world: &mut World, action: Action) -> Action {
        match action {
            Action::SelectEntries(file_entities) => {
                let mut system_state: SystemState<(
                    Commands,
                    Client,
                    Query<(Entity, &mut FileSystemUiState)>,
                    Query<(Entity, &ChangelistEntry, &mut ChangelistUiState)>,
                )> = SystemState::new(world);
                let (mut commands, mut client, mut fs_query, mut cl_query) =
                    system_state.get_mut(world);

                // TODO: when shift/control is pressed, select multiple items

                // Deselect all selected files, select the new selected files
                let (deselected_row_entities, mut file_entries_to_release) =
                    Self::deselect_all_selected_files(&mut client, &mut fs_query, &mut cl_query);
                let mut file_entries_to_request =
                    Self::select_files(&mut client, &mut fs_query, &mut cl_query, &file_entities);

                Self::remove_duplicates(&mut file_entries_to_release, &mut file_entries_to_request);

                Self::release_entities(&mut commands, &mut client, file_entries_to_release);
                Self::request_entities(&mut commands, &mut client, file_entries_to_request);

                system_state.apply(world);

                return Action::SelectEntries(deselected_row_entities);
            }
            Action::NewEntry(
                parent_entity_opt,
                new_file_name,
                entry_kind,
                old_entity_opt,
                entry_contents_opt,
            ) => {
                let mut system_state: SystemState<(
                    Commands,
                    Client,
                    Res<Global>,
                    ResMut<Canvas>,
                    ResMut<CameraManager>,
                    ResMut<VertexManager>,
                    ResMut<TabManager>,
                    Query<(Entity, &mut FileSystemUiState)>,
                    Query<(Entity, &ChangelistEntry, &mut ChangelistUiState)>,
                    Query<&mut FileSystemParent>,
                    Query<(&mut Visibility, &OwnedByTab)>,
                )> = SystemState::new(world);
                let (
                    mut commands,
                    mut client,
                    global,
                    mut canvas,
                    mut camera_manager,
                    mut vertex_manager,
                    mut tab_manager,
                    mut fs_query,
                    mut cl_query,
                    mut parent_query,
                    mut visibility_q,
                ) = system_state.get_mut(world);

                let (deselected_row_entities, file_entries_to_release) =
                    Self::deselect_all_selected_files(&mut client, &mut fs_query, &mut cl_query);
                Self::release_entities(&mut commands, &mut client, file_entries_to_release);

                let parent_entity = {
                    if let Some(parent_entity) = parent_entity_opt {
                        parent_entity
                    } else {
                        global.project_root_entity
                    }
                };

                // expand parent if it's not expanded
                {
                    if let Ok((_, mut fs_ui_state)) = fs_query.get_mut(parent_entity) {
                        fs_ui_state.opened = true;
                    }
                }

                // actually create new entry
                let mut parent = parent_query.get_mut(parent_entity).unwrap();

                let entity_id = self.create_fs_entry(
                    &mut commands,
                    &mut client,
                    &mut parent,
                    parent_entity_opt,
                    &new_file_name,
                    entry_kind,
                    entry_contents_opt,
                );

                // migrate undo entities
                if let Some(old_entity) = old_entity_opt {
                    self.migrate_file_entities(old_entity, entity_id);
                }

                // open tab for new entry
                tab_manager.open_tab(
                    &mut client,
                    &mut canvas,
                    &mut camera_manager,
                    &mut vertex_manager,
                    &mut visibility_q,
                    &entity_id,
                    FileExtension::from_file_name(&new_file_name),
                );

                system_state.apply(world);

                return Action::DeleteEntry(entity_id, Some(deselected_row_entities));
            }
            Action::DeleteEntry(file_entity, files_to_select_opt) => {
                let mut system_state: SystemState<(
                    Commands,
                    Client,
                    Res<Global>,
                    Query<(Entity, &mut FileSystemUiState)>,
                    Query<(Entity, &ChangelistEntry, &mut ChangelistUiState)>,
                    Query<(
                        &FileSystemEntry,
                        Option<&FileSystemChild>,
                        Option<&FileSystemRootChild>,
                    )>,
                    Query<&mut FileSystemParent>,
                )> = SystemState::new(world);
                let (
                    mut commands,
                    mut client,
                    global,
                    mut ui_query,
                    mut cl_query,
                    fs_query,
                    mut parent_query,
                ) = system_state.get_mut(world);
                let (entry, fs_child_opt, fs_root_child_opt) = fs_query.get(file_entity).unwrap();

                // get name of file
                let entry_name = entry.name.to_string();
                let entry_kind = *entry.kind;

                // get parent entity
                let parent_entity_opt: Option<Entity> = if let Some(fs_child) = fs_child_opt {
                    // get parent entity
                    let Some(parent_entity) = fs_child.parent_id.get(&client) else {
                        panic!("FileSystemChild {:?} has no parent!", file_entity);
                    };
                    // remove entity from parent
                    parent_query
                        .get_mut(parent_entity)
                        .unwrap()
                        .remove_child(&file_entity);

                    Some(parent_entity)
                } else if let Some(_) = fs_root_child_opt {
                    // remove entity from root
                    parent_query
                        .get_mut(global.project_root_entity)
                        .unwrap()
                        .remove_child(&file_entity);

                    None
                } else {
                    panic!(
                        "FileSystemEntry {:?} has neither FileSystemChild nor FileSystemRootChild!",
                        file_entity
                    );
                };

                let entry_contents_opt = {
                    match entry_kind {
                        EntryKind::File => None,
                        EntryKind::Directory => {
                            let entries = Self::convert_contents_to_slim_tree(
                                &client,
                                &file_entity,
                                &fs_query,
                                &mut parent_query,
                            );

                            Some(entries)
                        }
                    }
                };

                // actually delete the entry
                commands.entity(file_entity).despawn();

                // select files as needed
                if let Some(files_to_select) = files_to_select_opt {
                    let file_entries_to_request = Self::select_files(
                        &mut client,
                        &mut ui_query,
                        &mut cl_query,
                        &files_to_select,
                    );
                    Self::request_entities(&mut commands, &mut client, file_entries_to_request);
                }

                system_state.apply(world);

                return Action::NewEntry(
                    parent_entity_opt,
                    entry_name,
                    entry_kind,
                    Some(file_entity),
                    entry_contents_opt
                        .map(|entries| entries.into_iter().map(|(_, tree)| tree).collect()),
                );
            }
            Action::RenameEntry(file_entity, new_name) => {
                let mut system_state: SystemState<Query<&mut FileSystemEntry>> =
                    SystemState::new(world);
                let mut entry_query = system_state.get_mut(world);
                let Ok(mut file_entry) = entry_query.get_mut(file_entity) else {
                    panic!("Failed to get FileSystemEntry for row entity {:?}!", file_entity);
                };
                let old_name: String = file_entry.name.to_string();
                *file_entry.name = new_name.clone();

                system_state.apply(world);

                return Action::RenameEntry(file_entity, old_name);
            }
            Action::SelectVertex(vertex_2d_entity_opt) => {
                info!("SelectVertex({:?})", vertex_2d_entity_opt);

                let mut system_state: SystemState<(Commands, Client, ResMut<VertexManager>)> =
                    SystemState::new(world);
                let (mut commands, mut client, mut vertex_manager) = system_state.get_mut(world);

                // Deselect all selected vertices, select the new selected vertices
                let (deselected_entity, entity_to_release) =
                    Self::deselect_all_selected_vertices(&mut vertex_manager);
                let entity_to_request =
                    Self::select_vertex(&mut vertex_manager, vertex_2d_entity_opt.map(|v| v));

                if entity_to_request != entity_to_release {
                    if let Some(entity) = entity_to_release {
                        let mut entity_mut = commands.entity(entity);
                        if entity_mut.authority(&client).is_some() {
                            entity_mut.release_authority(&mut client);
                        }
                    }
                    if let Some(entity) = entity_to_request {
                        info!("request_entity({:?})", entity);
                        let mut entity_mut = commands.entity(entity);
                        if entity_mut.authority(&client).is_some() {
                            entity_mut.request_authority(&mut client);
                        }
                    }
                }

                system_state.apply(world);

                return Action::SelectVertex(deselected_entity);
            }
            Action::CreateVertex(vertex_type_data, position, old_vertex_entities_opt) => {
                let mut entities_to_release = Vec::new();
                let deselected_vertex_2d_entity_store;
                let selected_vertex_3d;
                let selected_vertex_2d;

                info!("CreateVertex");

                {
                    let mut system_state: SystemState<(
                        Commands,
                        Client,
                        ResMut<CameraManager>,
                        ResMut<VertexManager>,
                        Res<TabManager>,
                        ResMut<Assets<CpuMesh>>,
                        ResMut<Assets<CpuMaterial>>,
                    )> = SystemState::new(world);
                    let (
                        mut commands,
                        mut client,
                        mut camera_manager,
                        mut vertex_manager,
                        tab_manager,
                        mut meshes,
                        mut materials,
                    ) = system_state.get_mut(world);

                    // deselect all selected vertices
                    let (deselected_vertex_2d_entity, vertex_3d_entity_to_release) =
                        Self::deselect_all_selected_vertices(&mut vertex_manager);
                    deselected_vertex_2d_entity_store = deselected_vertex_2d_entity;
                    if let Some(entity) = vertex_3d_entity_to_release {
                        let mut entity_mut = commands.entity(entity);
                        if entity_mut.authority(&client).is_some() {
                            entity_mut.release_authority(&mut client);
                        }
                    }

                    // create vertex
                    let (new_vertex_2d_entity, new_vertex_3d_entity) = self.create_networked_vertex(
                        &mut commands,
                        &mut client,
                        &mut camera_manager,
                        &mut vertex_manager,
                        &mut meshes,
                        &mut materials,
                        position,
                        tab_manager.current_tab_id(),
                        vertex_type_data.to_file_type_value(),
                        &mut entities_to_release,
                    );
                    // migrate undo entities
                    if let Some((old_vertex_2d_entity, old_vertex_3d_entity)) =
                        old_vertex_entities_opt
                    {
                        self.migrate_vertex_entities(
                            old_vertex_2d_entity,
                            new_vertex_2d_entity,
                            old_vertex_3d_entity,
                            new_vertex_3d_entity,
                        );
                    }

                    if let Some(parent_vertex_2d_entity) = match vertex_type_data {
                        VertexTypeData::Skel(parent_vertex_2d_entity, children_opt) => {
                            if let Some(children) = children_opt {
                                self.create_networked_children_tree(
                                    &mut commands,
                                    &mut client,
                                    &mut camera_manager,
                                    &mut vertex_manager,
                                    &mut meshes,
                                    &mut materials,
                                    new_vertex_2d_entity,
                                    children,
                                    tab_manager.current_tab_id(),
                                    &mut entities_to_release,
                                );
                            }
                            Some(parent_vertex_2d_entity)
                        }
                        VertexTypeData::Mesh(parent_vertex_2d_entity_opt) => {
                            parent_vertex_2d_entity_opt
                        }
                    } {
                        self.create_networked_edge(
                            &mut commands,
                            &mut client,
                            &mut camera_manager,
                            &mut vertex_manager,
                            &mut meshes,
                            &mut materials,
                            parent_vertex_2d_entity,
                            new_vertex_2d_entity,
                            new_vertex_3d_entity,
                            tab_manager.current_tab_id(),
                            FileTypeValue::Mesh,
                            &mut entities_to_release,
                        );
                    }

                    // select vertex
                    vertex_manager.select_vertex(&new_vertex_2d_entity, CanvasShape::Vertex);
                    selected_vertex_3d = new_vertex_3d_entity;
                    selected_vertex_2d = new_vertex_2d_entity;

                    system_state.apply(world);
                }

                // release all non-selected vertices
                {
                    let mut system_state: SystemState<(Commands, Client)> = SystemState::new(world);
                    let (mut commands, mut client) = system_state.get_mut(world);

                    for entity_to_release in entities_to_release {
                        if entity_to_release != selected_vertex_3d {
                            commands
                                .entity(entity_to_release)
                                .release_authority(&mut client);
                        }
                    }

                    system_state.apply(world);
                }

                return Action::DeleteVertex(selected_vertex_2d, deselected_vertex_2d_entity_store);
            }
            Action::DeleteVertex(vertex_2d_entity, vertex_2d_to_select_opt) => {
                info!("DeleteVertex({:?})", vertex_2d_entity);

                let mut system_state: SystemState<(
                    Commands,
                    Client,
                    ResMut<VertexManager>,
                    Query<(Entity, &Vertex3d)>,
                    Query<&Edge3d>,
                    Query<&FileType>
                )> = SystemState::new(world);
                let (mut commands, mut client, mut vertex_manager, vertex_q, edge_3d_q, file_type_q) =
                    system_state.get_mut(world);

                let vertex_3d_entity = *vertex_manager
                    .vertex_entity_2d_to_3d(&vertex_2d_entity)
                    .unwrap();

                let Ok(file_type) = file_type_q.get(vertex_3d_entity) else {
                    panic!("Failed to get FileType for vertex entity {:?}!", vertex_3d_entity);
                };
                let file_type_value = *file_type.value;

                match file_type_value {
                    FileTypeValue::Skel => {
                        // get parent entity
                        let parent_vertex_2d_entity = {
                            let mut parent_vertex_3d_entity = None;
                            for edge_3d in edge_3d_q.iter() {
                                let Some(child_entity) = edge_3d.end.get(&client) else {
                                    continue;
                                };
                                let Some(parent_entity) = edge_3d.start.get(&client) else {
                                    continue;
                                };
                                if child_entity == vertex_3d_entity {
                                    parent_vertex_3d_entity = Some(parent_entity);
                                    break;
                                }
                            }
                            if parent_vertex_3d_entity.is_none() {
                                panic!(
                                    "Failed to find parent vertex for vertex entity {:?}!",
                                    vertex_3d_entity
                                );
                            }
                            *vertex_manager
                                .vertex_entity_3d_to_2d(&parent_vertex_3d_entity.unwrap())
                                .unwrap()
                        };

                        // get entries
                        let entry_contents_opt = {
                            let entries = Self::convert_vertices_to_tree(
                                &client,
                                &mut vertex_manager,
                                &vertex_3d_entity,
                                &vertex_q,
                                &edge_3d_q,
                            );

                            Some(entries)
                        };

                        let rev_vertex_type_data = VertexTypeData::Skel(
                            parent_vertex_2d_entity,
                            entry_contents_opt
                                .map(|entries| entries.into_iter().map(|(_, entry)| entry).collect()),
                        );

                        let Ok((_, vertex_3d)) = vertex_q.get(vertex_3d_entity) else {
                            panic!("Failed to get VertexChild for vertex entity {:?}!", vertex_3d_entity);
                        };
                        let vertex_3d_position = vertex_3d.as_vec3();

                        Self::handle_common_vertex_despawn(
                            &mut commands,
                            &mut client,
                            &mut vertex_manager,
                            vertex_3d_entity,
                            vertex_2d_to_select_opt,
                        );

                        system_state.apply(world);

                        return Action::CreateVertex(
                            rev_vertex_type_data,
                            vertex_3d_position,
                            Some((vertex_2d_entity, vertex_3d_entity)),
                        );
                    }
                    FileTypeValue::Mesh => {

                        // TODO: handle vertex connections

                        let rev_vertex_type_data = VertexTypeData::Mesh(
                            None,
                        );

                        let Ok((_, vertex_3d)) = vertex_q.get(vertex_3d_entity) else {
                            panic!("Failed to get VertexChild for vertex entity {:?}!", vertex_3d_entity);
                        };
                        let vertex_3d_position = vertex_3d.as_vec3();

                        Self::handle_common_vertex_despawn(
                            &mut commands,
                            &mut client,
                            &mut vertex_manager,
                            vertex_3d_entity,
                            vertex_2d_to_select_opt,
                        );

                        system_state.apply(world);

                        return Action::CreateVertex(
                            rev_vertex_type_data,
                            vertex_3d_position,
                            Some((vertex_2d_entity, vertex_3d_entity)),
                        );
                    }
                }
            }
            Action::MoveVertex(vertex_2d_entity, old_position, new_position) => {
                info!("MoveVertex");
                let mut system_state: SystemState<(
                    ResMut<VertexManager>,
                    ResMut<CameraManager>,
                    Query<&mut Vertex3d>,
                )> = SystemState::new(world);
                let (vertex_manager, mut camera_manager, mut vertex_3d_q) =
                    system_state.get_mut(world);

                let vertex_3d_entity = *vertex_manager
                    .vertex_entity_2d_to_3d(&vertex_2d_entity)
                    .unwrap();

                let Ok(mut vertex_3d) = vertex_3d_q.get_mut(vertex_3d_entity) else {
                    panic!("Failed to get Vertex3d for vertex entity {:?}!", vertex_3d_entity);
                };
                vertex_3d.set_vec3(&new_position);

                camera_manager.recalculate_3d_view();

                system_state.apply(world);

                return Action::MoveVertex(vertex_2d_entity, new_position, old_position);
            }
        }
    }

    fn handle_common_vertex_despawn(
        commands: &mut Commands,
        client: &mut Client,
        vertex_manager: &mut VertexManager,
        vertex_3d_entity: Entity,
        vertex_2d_to_select_opt: Option<(Entity, CanvasShape)>,
    ) {
        // delete 3d vertex
        commands.entity(vertex_3d_entity).despawn();

        // cleanup mappings
        vertex_manager.cleanup_deleted_vertex(&vertex_3d_entity, commands);

        // select entities as needed
        if let Some((vertex_2d_to_select, vertex_type)) = vertex_2d_to_select_opt {
            if let Some(vertex_3d_entity_to_request) =
                Self::select_vertex(vertex_manager, Some((vertex_2d_to_select, vertex_type)))
            {
                info!("request_entities({:?})", vertex_3d_entity_to_request);
                let mut entity_mut = commands.entity(vertex_3d_entity_to_request);
                if entity_mut.authority(client).is_some() {
                    entity_mut.request_authority(client);
                }
            }
        } else {
            vertex_manager.deselect_vertex();
        }
    }

    fn select_files(
        client: &mut Client,
        fs_query: &mut Query<(Entity, &mut FileSystemUiState)>,
        cl_query: &mut Query<(Entity, &ChangelistEntry, &mut ChangelistUiState)>,
        row_entities: &Vec<Entity>,
    ) -> HashSet<Entity> {
        let mut file_entries_to_request = HashSet::new();
        for row_entity in row_entities {
            if let Ok((_, mut ui_state)) = fs_query.get_mut(*row_entity) {
                // File System
                ui_state.selected = true;

                file_entries_to_request.insert(*row_entity);
            }
            if let Ok((_, cl_entry, mut ui_state)) = cl_query.get_mut(*row_entity) {
                // Changelist
                ui_state.selected = true;

                if let Some(file_entity) = cl_entry.file_entity.get(client) {
                    file_entries_to_request.insert(file_entity);
                }
            }
        }
        file_entries_to_request
    }

    fn deselect_all_selected_files(
        client: &mut Client,
        fs_query: &mut Query<(Entity, &mut FileSystemUiState)>,
        cl_query: &mut Query<(Entity, &ChangelistEntry, &mut ChangelistUiState)>,
    ) -> (Vec<Entity>, HashSet<Entity>) {
        let mut deselected_row_entities = Vec::new();
        let mut file_entries_to_release = HashSet::new();
        for (item_entity, mut ui_state) in fs_query.iter_mut() {
            // FileSystem
            if ui_state.selected {
                ui_state.selected = false;

                deselected_row_entities.push(item_entity);
                file_entries_to_release.insert(item_entity);
            }
        }
        for (item_entity, cl_entry, mut ui_state) in cl_query.iter_mut() {
            // Changelist
            if ui_state.selected {
                ui_state.selected = false;

                deselected_row_entities.push(item_entity);

                if let Some(file_entity) = cl_entry.file_entity.get(client) {
                    file_entries_to_release.insert(file_entity);
                }
            }
        }
        (deselected_row_entities, file_entries_to_release)
    }

    // returns entity to request auth for
    fn select_vertex(
        vertex_manager: &mut VertexManager,
        vertex_2d_entity_opt: Option<(Entity, CanvasShape)>,
    ) -> Option<Entity> {
        if let Some((vertex_2d_entity, shape)) = vertex_2d_entity_opt {
            vertex_manager.select_vertex(&vertex_2d_entity, shape);
            if shape == CanvasShape::Vertex {
                let vertex_3d_entity = vertex_manager
                    .vertex_entity_2d_to_3d(&vertex_2d_entity)
                    .unwrap();
                return Some(*vertex_3d_entity);
            } else {
                return None;
            }
        }
        return None;
    }

    fn deselect_all_selected_vertices(
        vertex_manager: &mut VertexManager,
    ) -> (Option<(Entity, CanvasShape)>, Option<Entity>) {
        let mut entity_to_deselect = None;
        let mut entity_to_release = None;
        if let Some((vertex_2d_entity, vertex_2d_type)) = vertex_manager.selected_vertex_2d() {
            vertex_manager.deselect_vertex();
            entity_to_deselect = Some((vertex_2d_entity, vertex_2d_type));
            if vertex_2d_type == CanvasShape::Vertex {
                let vertex_3d_entity = vertex_manager
                    .vertex_entity_2d_to_3d(&vertex_2d_entity)
                    .unwrap();
                entity_to_release = Some(*vertex_3d_entity);
            }
        }
        (entity_to_deselect, entity_to_release)
    }

    fn request_entities(
        commands: &mut Commands,
        client: &mut Client,
        entities_to_request: HashSet<Entity>,
    ) {
        for file_entity in entities_to_request {
            info!("request_entities({:?})", file_entity);
            let mut entity_mut = commands.entity(file_entity);
            if entity_mut.authority(client).is_some() {
                entity_mut.request_authority(client);
            }
        }
    }

    fn release_entities(
        commands: &mut Commands,
        client: &mut Client,
        entities_to_release: HashSet<Entity>,
    ) {
        for file_entity in entities_to_release {
            let mut entity_mut = commands.entity(file_entity);
            if entity_mut.authority(client).is_some() {
                entity_mut.release_authority(client);
            }
        }
    }

    fn remove_duplicates(set_a: &mut HashSet<Entity>, set_b: &mut HashSet<Entity>) {
        set_a.retain(|item| {
            if set_b.contains(item) {
                set_b.remove(item);
                false // Remove the item from set_a
            } else {
                true // Keep the item in set_a
            }
        });
    }

    fn create_fs_entry(
        &mut self,
        commands: &mut Commands,
        client: &mut Client,
        parent: &mut FileSystemParent,
        parent_entity_opt: Option<Entity>,
        new_file_name: &str,
        entry_kind: EntryKind,
        entry_contents_opt: Option<Vec<FileTree>>,
    ) -> Entity {
        info!("creating new entry: `{}`", new_file_name);

        let entity_id = commands
            .spawn_empty()
            .enable_replication(client)
            .configure_replication(ReplicationConfig::Delegated)
            .id();

        let entry = FileSystemEntry::new(new_file_name, entry_kind);

        // add FileSystemChild or FileSystemRootChild component
        if let Some(parent_entity) = parent_entity_opt {
            let mut child_component = FileSystemChild::new();
            child_component.parent_id.set(client, &parent_entity);
            commands.entity(entity_id).insert(child_component);
        } else {
            commands.entity(entity_id).insert(FileSystemRootChild);
        }

        // add UiState component
        file_post_process::insert_ui_state_component(commands, entity_id, true);

        if *entry.kind == EntryKind::Directory {
            let mut entry_parent_component = FileSystemParent::new();

            if let Some(entry_contents) = entry_contents_opt {
                for sub_tree in entry_contents {
                    let new_entity = self.create_fs_entry(
                        commands,
                        client,
                        &mut entry_parent_component,
                        Some(entity_id),
                        &sub_tree.name,
                        sub_tree.kind,
                        sub_tree.children,
                    );
                    let old_entity = sub_tree.entity;
                    self.migrate_file_entities(old_entity, new_entity);
                }
            }

            // add FileSystemParent component
            commands.entity(entity_id).insert(entry_parent_component);
        }

        // add child to parent
        file_post_process::parent_add_child_entry(parent, &entry, entity_id);

        // add FileSystemEntry component
        commands.entity(entity_id).insert(entry);

        entity_id
    }

    fn create_networked_vertex(
        &mut self,
        commands: &mut Commands,
        client: &mut Client,
        camera_manager: &mut CameraManager,
        vertex_manager: &mut VertexManager,
        meshes: &mut Assets<CpuMesh>,
        materials: &mut Assets<CpuMaterial>,
        position: Vec3,
        tab_id: TabId,
        file_type: FileTypeValue,
        entities_to_release: &mut Vec<Entity>,
    ) -> (Entity, Entity) {

        // create new 3d vertex
        let new_vertex_3d_entity = commands
            .spawn_empty()
            .enable_replication(client)
            .configure_replication(ReplicationConfig::Delegated)
            .insert(Vertex3d::from_vec3(position))
            .insert(OwnedByTab::new(tab_id))
            .insert(FileType::new(file_type))
            .id();

        entities_to_release.push(new_vertex_3d_entity);

        // create new 2d vertex, add local components to 3d vertex
        let new_vertex_2d_entity = vertex_manager.vertex_3d_postprocess(
            commands,
            meshes,
            materials,
            camera_manager,
            new_vertex_3d_entity,
            false,
            Some(tab_id),
            Vertex2d::CHILD_COLOR,
        );

        return (new_vertex_2d_entity, new_vertex_3d_entity);
    }

    fn create_networked_edge(
        &mut self,
        commands: &mut Commands,
        client: &mut Client,
        camera_manager: &mut CameraManager,
        vertex_manager: &mut VertexManager,
        meshes: &mut Assets<CpuMesh>,
        materials: &mut Assets<CpuMaterial>,
        parent_vertex_2d_entity: Entity,
        child_vertex_2d_entity: Entity,
        child_vertex_3d_entity: Entity,
        tab_id: TabId,
        file_type: FileTypeValue,
        entities_to_release: &mut Vec<Entity>,
    ) {
        // create new 3d edge
        let parent_vertex_3d_entity = *vertex_manager
            .vertex_entity_2d_to_3d(&parent_vertex_2d_entity)
            .unwrap();

        let mut new_edge_3d_component = Edge3d::new();
        new_edge_3d_component
            .start
            .set(client, &parent_vertex_3d_entity);
        new_edge_3d_component.end.set(client, &child_vertex_3d_entity);
        let new_edge_3d_entity = commands
            .spawn_empty()
            .enable_replication(client)
            .configure_replication(ReplicationConfig::Delegated)
            .insert(new_edge_3d_component)
            .insert(Edge3dLocal::new(
                parent_vertex_3d_entity,
                child_vertex_3d_entity,
            ))
            .insert(OwnedByTab::new(tab_id))
            .insert(FileType::new(file_type))
            .id();

        // create new 2d edge, add local components to 3d edge
        let _new_edge_2d_entity = vertex_manager.edge_3d_postprocess(
            commands,
            meshes,
            materials,
            camera_manager,
            new_edge_3d_entity,
            child_vertex_2d_entity,
            parent_vertex_2d_entity,
            Some(tab_id),
            Vertex2d::CHILD_COLOR,
            file_type == FileTypeValue::Skel,
        );

        entities_to_release.push(new_edge_3d_entity);
    }

    fn create_networked_children_tree(
        &mut self,
        commands: &mut Commands,
        client: &mut Client,
        camera_manager: &mut CameraManager,
        vertex_manager: &mut VertexManager,
        meshes: &mut Assets<CpuMesh>,
        materials: &mut Assets<CpuMaterial>,
        parent_vertex_2d_entity: Entity,
        children: Vec<VertexEntry>,
        tab_id: TabId,
        entities_to_release: &mut Vec<Entity>,
    ) {
        for child in children {
            let position = child.position();
            let grandchildren_opt = child.children();
            let old_child_vertex_3d_entity = child.entity_3d();
            let old_child_vertex_2d_entity = child.entity_2d();

            let (new_child_vertex_2d_entity, new_child_vertex_3d_entity) = self
                .create_networked_vertex(
                    commands,
                    client,
                    camera_manager,
                    vertex_manager,
                    meshes,
                    materials,
                    position,
                    tab_id,
                    FileTypeValue::Skel,
                    entities_to_release,
                );
            self.migrate_vertex_entities(
                old_child_vertex_2d_entity,
                new_child_vertex_2d_entity,
                old_child_vertex_3d_entity,
                new_child_vertex_3d_entity,
            );
            self.create_networked_edge(
                commands,
                client,
                camera_manager,
                vertex_manager,
                meshes,
                materials,
                parent_vertex_2d_entity,
                new_child_vertex_2d_entity,
                new_child_vertex_3d_entity,
                tab_id,
                FileTypeValue::Skel,
                entities_to_release,
            );
            if let Some(grandchildren) = grandchildren_opt {
                self.create_networked_children_tree(
                    commands,
                    client,
                    camera_manager,
                    vertex_manager,
                    meshes,
                    materials,
                    new_child_vertex_2d_entity,
                    grandchildren,
                    tab_id,
                    entities_to_release
                );
            }
        }
    }

    pub fn entity_update_auth_status(
        &mut self,
        vertex_manager: &mut VertexManager,
        entity: &Entity,
    ) {
        // if either the undo or redo stack's top entity is this entity, then we need to enable/disable undo based on new auth status
        Self::entity_update_auth_status_impl(
            vertex_manager,
            &mut self.buffered_check,
            self.undo_actions.last(),
            entity,
        );
        Self::entity_update_auth_status_impl(
            vertex_manager,
            &mut self.buffered_check,
            self.redo_actions.last(),
            entity,
        );
    }

    fn entity_update_auth_status_impl(
        vertex_manager: &mut VertexManager,
        buffered_check: &mut bool,
        action_opt: Option<&Action>,
        entity: &Entity,
    ) {
        match action_opt {
            Some(Action::SelectEntries(file_entities)) => {
                if file_entities.contains(entity) {
                    *buffered_check = true;
                }
            }
            Some(Action::SelectVertex(vertex_2d_entity_opt)) => {
                if let Some((vertex_2d_entity, CanvasShape::Vertex)) = vertex_2d_entity_opt {
                    let vertex_3d_entity = vertex_manager
                        .vertex_entity_2d_to_3d(vertex_2d_entity)
                        .unwrap();
                    if *vertex_3d_entity == *entity {
                        *buffered_check = true;
                    }
                }
            }
            _ => {}
        }
    }

    fn enable_top(&mut self, world: &mut World) {
        Self::enable_top_impl(world, self.undo_actions.last(), &mut self.undo_enabled);
        Self::enable_top_impl(world, self.redo_actions.last(), &mut self.redo_enabled);

        //info!("undo enabled: {}", self.undo_enabled);
    }

    fn enable_top_impl(world: &mut World, last_action: Option<&Action>, enabled: &mut bool) {
        match last_action {
            Some(Action::SelectEntries(entities)) => {
                *enabled = Self::should_be_enabled(world, entities);
            }
            Some(Action::SelectVertex(vertex_2d_entity_opt)) => {
                let mut entities = Vec::new();

                if let Some((vertex_2d_entity, CanvasShape::Vertex)) = vertex_2d_entity_opt {
                    let vertex_3d_entity = world
                        .get_resource::<VertexManager>()
                        .unwrap()
                        .vertex_entity_2d_to_3d(vertex_2d_entity)
                        .unwrap();
                    entities.push(*vertex_3d_entity);
                }

                *enabled = Self::should_be_enabled(world, &entities);
            }
            _ => {
                *enabled = true;
            }
        }
    }

    fn should_be_enabled(world: &mut World, entities: &Vec<Entity>) -> bool {
        let mut system_state: SystemState<(Commands, Client)> = SystemState::new(world);
        let (mut commands, client) = system_state.get_mut(world);

        for file_entity in entities {
            if let Some(EntityAuthStatus::Available) =
                commands.entity(*file_entity).authority(&client)
            {
                // enabled should continue being true
            } else {
                return false;
            }
        }
        return true;
    }

    fn convert_contents_to_slim_tree(
        client: &Client,
        parent_entity: &Entity,
        fs_query: &Query<(
            &FileSystemEntry,
            Option<&FileSystemChild>,
            Option<&FileSystemRootChild>,
        )>,
        parent_query: &mut Query<&mut FileSystemParent>,
    ) -> Vec<(Entity, FileTree)> {
        let mut trees = Vec::new();

        if let Ok(parent) = parent_query.get(*parent_entity) {
            let children_entities = parent.get_children();
            for child_entity in children_entities {
                let (child_entry, _, _) = fs_query.get(child_entity).unwrap();
                let slim_tree = FileTree::new(
                    child_entity,
                    child_entry.name.to_string(),
                    *child_entry.kind,
                );
                trees.push((child_entity, slim_tree));
            }

            for (entry_entity, tree) in trees.iter_mut() {
                let subtree = Self::convert_contents_to_slim_tree(
                    client,
                    entry_entity,
                    fs_query,
                    parent_query,
                );
                if subtree.len() > 0 {
                    tree.children = Some(
                        subtree
                            .into_iter()
                            .map(|(_, child_tree)| child_tree)
                            .collect(),
                    );
                }
            }
        }

        trees
    }

    fn convert_vertices_to_tree(
        client: &Client,
        vertex_manager: &mut VertexManager,
        parent_3d_entity: &Entity,
        vertex_3d_q: &Query<(Entity, &Vertex3d)>,
        edge_3d_q: &Query<&Edge3d>,
    ) -> Vec<(Entity, VertexEntry)> {
        let mut output = Vec::new();

        for edge_3d in edge_3d_q.iter() {
            let Some(parent_entity) = edge_3d.start.get(client) else {
                warn!("edge start not found");
                continue;
            };
            let Some(child_entity_3d) = edge_3d.end.get(client) else {
                warn!("edge end not found");
                continue;
            };
            if parent_entity == *parent_3d_entity {
                let child_entity_2d = *vertex_manager
                    .vertex_entity_3d_to_2d(&child_entity_3d)
                    .unwrap();

                // get positon
                let Ok((_, vertex_3d)) = vertex_3d_q.get(child_entity_3d) else {
                    panic!("vertex entity not found");
                };

                let child_entry =
                    VertexEntry::new(child_entity_2d, child_entity_3d, vertex_3d.as_vec3());
                output.push((child_entity_3d, child_entry));
            }
        }

        for (entry_entity, entry) in output.iter_mut() {
            // set children
            let children = Self::convert_vertices_to_tree(
                client,
                vertex_manager,
                entry_entity,
                vertex_3d_q,
                edge_3d_q,
            );
            if children.len() > 0 {
                entry.set_children(
                    children
                        .into_iter()
                        .map(|(_, child_tree)| child_tree)
                        .collect(),
                );
            }
        }

        output
    }

    fn migrate_file_entities(&mut self, old_entity: Entity, new_entity: Entity) {
        for action in self.undo_actions.iter_mut() {
            action.migrate_file_entities(old_entity, new_entity);
        }
        for action in self.redo_actions.iter_mut() {
            action.migrate_file_entities(old_entity, new_entity);
        }
    }

    fn migrate_vertex_entities(
        &mut self,
        old_2d_entity: Entity,
        new_2d_entity: Entity,
        old_3d_entity: Entity,
        new_3d_entity: Entity,
    ) {
        for action in self.undo_actions.iter_mut() {
            action.migrate_vertex_entities(
                old_2d_entity,
                new_2d_entity,
                old_3d_entity,
                new_3d_entity,
            );
        }
        for action in self.redo_actions.iter_mut() {
            action.migrate_vertex_entities(
                old_2d_entity,
                new_2d_entity,
                old_3d_entity,
                new_3d_entity,
            );
        }
    }
}
