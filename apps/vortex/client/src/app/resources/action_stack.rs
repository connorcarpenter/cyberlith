use std::collections::HashSet;

use bevy_ecs::{
    prelude::{Commands, Entity, Query, Resource, World},
    system::SystemState,
};
use bevy_log::{info, warn};

use naia_bevy_client::{Client, CommandsExt, EntityAuthStatus, ReplicationConfig};

use math::Vec3;
use render_api::{
    base::{CpuMaterial, CpuMesh},
    Assets,
};

use vortex_proto::components::{
    ChangelistEntry, Edge3d, EntryKind, FileSystemChild, FileSystemEntry, FileSystemRootChild,
    FileType, FileTypeValue, OwnedByFile, Vertex3d,
};

use crate::app::{
    components::{
        file_system::{ChangelistUiState, FileSystemParent, FileSystemUiState},
        Vertex2d, VertexEntry,
    },
    resources::{
        action::Action,
        camera_manager::CameraManager,
        file_tree::FileTree,
        shape_manager::{CanvasShape, ShapeManager},
    },
    systems::file_post_process,
};

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

        let mut reversed_actions = self.execute_action(world, action);

        self.redo_actions.append(&mut reversed_actions);

        self.enable_top(world);
    }

    pub fn redo_action(&mut self, world: &mut World) {
        if !self.redo_enabled {
            panic!("Redo is disabled!");
        }
        let Some(action) = self.redo_actions.pop() else {
            panic!("No undone actions to redo!");
        };

        let mut reversed_actions = self.execute_action(world, action);

        self.undo_actions.append(&mut reversed_actions);

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
            let mut reversed_actions = self.execute_action(world, action);
            self.undo_actions.append(&mut reversed_actions);
        }
        self.redo_actions.clear();

        self.enable_top(world);
    }

    fn execute_action(&mut self, world: &mut World, action: Action) -> Vec<Action> {
        action.execute(world, self)
    }

    pub(crate) fn handle_common_vertex_despawn(
        commands: &mut Commands,
        client: &mut Client,
        shape_manager: &mut ShapeManager,
        vertex_3d_entity: Entity,
        vertex_2d_to_select_opt: Option<(Entity, CanvasShape)>,
    ) {
        // delete 3d vertex
        commands.entity(vertex_3d_entity).despawn();

        // cleanup mappings
        shape_manager.cleanup_deleted_vertex(commands, &vertex_3d_entity);

        // select entities as needed
        if let Some((vertex_2d_to_select, vertex_type)) = vertex_2d_to_select_opt {
            if let Some(vertex_3d_entity_to_request) =
                Self::select_shape(shape_manager, Some((vertex_2d_to_select, vertex_type)))
            {
                //info!("request_entities({:?})", vertex_3d_entity_to_request);
                let mut entity_mut = commands.entity(vertex_3d_entity_to_request);
                if entity_mut.authority(client).is_some() {
                    entity_mut.request_authority(client);
                }
            }
        } else {
            shape_manager.deselect_shape();
        }
    }

    pub(crate) fn select_files(
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

    pub(crate) fn deselect_all_selected_files(
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
    pub(crate) fn select_shape(
        shape_manager: &mut ShapeManager,
        shape_2d_entity_opt: Option<(Entity, CanvasShape)>,
    ) -> Option<Entity> {
        if let Some((shape_2d_entity, shape)) = shape_2d_entity_opt {
            shape_manager.select_shape(&shape_2d_entity, shape);
            match shape {
                CanvasShape::Vertex => {
                    let vertex_3d_entity = shape_manager
                        .vertex_entity_2d_to_3d(&shape_2d_entity)
                        .unwrap();
                    return Some(vertex_3d_entity);
                }
                CanvasShape::Edge => {
                    let edge_3d_entity = shape_manager
                        .edge_entity_2d_to_3d(&shape_2d_entity)
                        .unwrap();
                    return Some(edge_3d_entity);
                }
                CanvasShape::Face => {
                    return shape_manager.face_entity_2d_to_3d(&shape_2d_entity);
                }
                _ => return None,
            }
        }
        return None;
    }

    pub(crate) fn deselect_all_selected_shapes(
        shape_manager: &mut ShapeManager,
    ) -> (Option<(Entity, CanvasShape)>, Option<Entity>) {
        let mut entity_to_deselect = None;
        let mut entity_to_release = None;
        if let Some((shape_2d_entity, shape_2d_type)) = shape_manager.selected_shape_2d() {
            shape_manager.deselect_shape();
            entity_to_deselect = Some((shape_2d_entity, shape_2d_type));
            match shape_2d_type {
                CanvasShape::Vertex => {
                    let vertex_3d_entity = shape_manager
                        .vertex_entity_2d_to_3d(&shape_2d_entity)
                        .unwrap();
                    entity_to_release = Some(vertex_3d_entity);
                }
                CanvasShape::Edge => {
                    let edge_3d_entity = shape_manager
                        .edge_entity_2d_to_3d(&shape_2d_entity)
                        .unwrap();
                    entity_to_release = Some(edge_3d_entity);
                }
                CanvasShape::Face => {
                    if let Some(face_3d_entity) =
                        shape_manager.face_entity_2d_to_3d(&shape_2d_entity)
                    {
                        entity_to_release = Some(face_3d_entity);
                    }
                }
                _ => {}
            }
        }
        (entity_to_deselect, entity_to_release)
    }

    pub(crate) fn request_entities(
        commands: &mut Commands,
        client: &mut Client,
        entities_to_request: HashSet<Entity>,
    ) {
        for file_entity in entities_to_request {
            //info!("request_entities({:?})", file_entity);
            let mut entity_mut = commands.entity(file_entity);
            if entity_mut.authority(client).is_some() {
                entity_mut.request_authority(client);
            }
        }
    }

    pub(crate) fn release_entities(
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

    pub(crate) fn remove_duplicates(set_a: &mut HashSet<Entity>, set_b: &mut HashSet<Entity>) {
        set_a.retain(|item| {
            if set_b.contains(item) {
                set_b.remove(item);
                false // Remove the item from set_a
            } else {
                true // Keep the item in set_a
            }
        });
    }

    pub(crate) fn create_fs_entry(
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

    pub(crate) fn create_networked_vertex(
        &mut self,
        commands: &mut Commands,
        client: &mut Client,
        camera_manager: &mut CameraManager,
        shape_manager: &mut ShapeManager,
        meshes: &mut Assets<CpuMesh>,
        materials: &mut Assets<CpuMaterial>,
        position: Vec3,
        file_entity: Entity,
        file_type: FileTypeValue,
        entities_to_release: &mut Vec<Entity>,
    ) -> (Entity, Entity) {
        // create new 3d vertex
        let mut owned_by_file_component = OwnedByFile::new();
        owned_by_file_component
            .file_entity
            .set(client, &file_entity);
        let new_vertex_3d_entity = commands
            .spawn_empty()
            .enable_replication(client)
            .configure_replication(ReplicationConfig::Delegated)
            .insert(Vertex3d::from_vec3(position))
            .insert(owned_by_file_component)
            .insert(FileType::new(file_type))
            .id();

        entities_to_release.push(new_vertex_3d_entity);

        // create new 2d vertex, add local components to 3d vertex
        let new_vertex_2d_entity = shape_manager.vertex_3d_postprocess(
            commands,
            meshes,
            materials,
            camera_manager,
            new_vertex_3d_entity,
            false,
            Some(file_entity),
            Vertex2d::CHILD_COLOR,
        );

        return (new_vertex_2d_entity, new_vertex_3d_entity);
    }

    // return (new edge 2d entity, new edge 3d entity)
    pub(crate) fn create_networked_edge(
        &mut self,
        commands: &mut Commands,
        client: &mut Client,
        camera_manager: &mut CameraManager,
        shape_manager: &mut ShapeManager,
        meshes: &mut Assets<CpuMesh>,
        materials: &mut Assets<CpuMaterial>,
        parent_vertex_2d_entity: Entity,
        child_vertex_2d_entity: Entity,
        child_vertex_3d_entity: Entity,
        file_entity: Entity,
        file_type: FileTypeValue,
        entities_to_release: &mut Vec<Entity>,
    ) -> (Entity, Entity) {
        // create new 3d edge
        let parent_vertex_3d_entity = shape_manager
            .vertex_entity_2d_to_3d(&parent_vertex_2d_entity)
            .unwrap();

        let mut new_edge_3d_component = Edge3d::new();
        new_edge_3d_component
            .start
            .set(client, &parent_vertex_3d_entity);
        new_edge_3d_component
            .end
            .set(client, &child_vertex_3d_entity);
        let mut owned_by_file_component = OwnedByFile::new();
        owned_by_file_component
            .file_entity
            .set(client, &file_entity);
        let new_edge_3d_entity = commands
            .spawn_empty()
            .enable_replication(client)
            .configure_replication(ReplicationConfig::Delegated)
            .insert(new_edge_3d_component)
            .insert(owned_by_file_component)
            .insert(FileType::new(file_type))
            .id();

        // create new 2d edge, add local components to 3d edge
        let new_edge_2d_entity = shape_manager.edge_3d_postprocess(
            commands,
            meshes,
            materials,
            camera_manager,
            new_edge_3d_entity,
            parent_vertex_2d_entity,
            parent_vertex_3d_entity,
            child_vertex_2d_entity,
            child_vertex_3d_entity,
            Some(file_entity),
            Vertex2d::CHILD_COLOR,
            file_type == FileTypeValue::Skel,
        );

        entities_to_release.push(new_edge_3d_entity);

        (new_edge_2d_entity, new_edge_3d_entity)
    }

    pub(crate) fn create_networked_children_tree(
        &mut self,
        commands: &mut Commands,
        client: &mut Client,
        camera_manager: &mut CameraManager,
        shape_manager: &mut ShapeManager,
        meshes: &mut Assets<CpuMesh>,
        materials: &mut Assets<CpuMaterial>,
        parent_vertex_2d_entity: Entity,
        children: Vec<VertexEntry>,
        file_entity: Entity,
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
                    shape_manager,
                    meshes,
                    materials,
                    position,
                    file_entity,
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
                shape_manager,
                meshes,
                materials,
                parent_vertex_2d_entity,
                new_child_vertex_2d_entity,
                new_child_vertex_3d_entity,
                file_entity,
                FileTypeValue::Skel,
                entities_to_release,
            );
            if let Some(grandchildren) = grandchildren_opt {
                self.create_networked_children_tree(
                    commands,
                    client,
                    camera_manager,
                    shape_manager,
                    meshes,
                    materials,
                    new_child_vertex_2d_entity,
                    grandchildren,
                    file_entity,
                    entities_to_release,
                );
            }
        }
    }

    pub fn entity_update_auth_status(&mut self, shape_manager: &mut ShapeManager, entity: &Entity) {
        // if either the undo or redo stack's top entity is this entity, then we need to enable/disable undo based on new auth status
        Self::entity_update_auth_status_impl(
            shape_manager,
            &mut self.buffered_check,
            self.undo_actions.last(),
            entity,
        );
        Self::entity_update_auth_status_impl(
            shape_manager,
            &mut self.buffered_check,
            self.redo_actions.last(),
            entity,
        );
    }

    fn entity_update_auth_status_impl(
        shape_manager: &mut ShapeManager,
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
            Some(Action::SelectShape(vertex_2d_entity_opt)) => {
                if let Some((vertex_2d_entity, CanvasShape::Vertex)) = vertex_2d_entity_opt {
                    let vertex_3d_entity = shape_manager
                        .vertex_entity_2d_to_3d(vertex_2d_entity)
                        .unwrap();
                    if vertex_3d_entity == *entity {
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
            Some(Action::SelectShape(vertex_2d_entity_opt)) => {
                let mut entities = Vec::new();

                if let Some((vertex_2d_entity, CanvasShape::Vertex)) = vertex_2d_entity_opt {
                    let vertex_3d_entity = world
                        .get_resource::<ShapeManager>()
                        .unwrap()
                        .vertex_entity_2d_to_3d(vertex_2d_entity)
                        .unwrap();
                    entities.push(vertex_3d_entity);
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

    pub(crate) fn convert_contents_to_slim_tree(
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

    pub(crate) fn convert_vertices_to_tree(
        client: &Client,
        shape_manager: &mut ShapeManager,
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
                let child_entity_2d = shape_manager
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
                shape_manager,
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

    pub(crate) fn migrate_file_entities(&mut self, old_entity: Entity, new_entity: Entity) {
        for action_list in [&mut self.undo_actions, &mut self.redo_actions] {
            for action in action_list.iter_mut() {
                action.migrate_file_entities(old_entity, new_entity);
            }
        }
    }

    pub(crate) fn migrate_vertex_entities(
        &mut self,
        old_2d_entity: Entity,
        new_2d_entity: Entity,
        old_3d_entity: Entity,
        new_3d_entity: Entity,
    ) {
        for action_list in [&mut self.undo_actions, &mut self.redo_actions] {
            for action in action_list.iter_mut() {
                action.migrate_vertex_entities(
                    old_2d_entity,
                    new_2d_entity,
                    old_3d_entity,
                    new_3d_entity,
                );
            }
        }
    }

    pub(crate) fn migrate_edge_entities(&mut self, old_2d_entity: Entity, new_2d_entity: Entity) {
        for action_list in [&mut self.undo_actions, &mut self.redo_actions] {
            for action in action_list.iter_mut() {
                action.migrate_edge_entities(old_2d_entity, new_2d_entity);
            }
        }
    }

    pub(crate) fn migrate_face_entities(&mut self, old_2d_entity: Entity, new_2d_entity: Entity) {
        info!(
            "migrate_face_entities({:?}, {:?})",
            old_2d_entity, new_2d_entity
        );
        for action_list in [&mut self.undo_actions, &mut self.redo_actions] {
            for action in action_list.iter_mut() {
                action.migrate_face_entities(old_2d_entity, new_2d_entity);
            }
        }
    }

    // pub fn remove_vertex_entity(&mut self, entity_2d: Entity, entity_3d: Entity) {
    //     for list in [&mut self.undo_actions, &mut self.redo_actions] {
    //         let mut new_actions = Vec::new();
    //         for mut action in std::mem::take(list) {
    //             if action.remove_vertex_entity(entity_2d, entity_3d) {
    //                 // don't add to new list
    //             } else {
    //                 new_actions.push(action);
    //             }
    //         }
    //         *list = new_actions;
    //     }
    // }
    //
    // pub fn remove_edge_entity(&mut self, entity_2d: Entity, entity_3d: Entity) {
    //     for list in [&mut self.undo_actions, &mut self.redo_actions] {
    //         let mut new_actions = Vec::new();
    //         for mut action in std::mem::take(list) {
    //             if action.remove_edge_entity(entity_2d, entity_3d) {
    //                 // don't add to new list
    //             } else {
    //                 new_actions.push(action);
    //             }
    //         }
    //         *list = new_actions;
    //     }
    // }
    //
    // pub fn remove_face_entity(&mut self, entity_2d: Entity) {
    //     for list in [&mut self.undo_actions, &mut self.redo_actions] {
    //         let mut new_actions = Vec::new();
    //         for mut action in std::mem::take(list) {
    //             if action.remove_face_entity(entity_2d) {
    //                 // don't add to new list
    //             } else {
    //                 new_actions.push(action);
    //             }
    //         }
    //         *list = new_actions;
    //     }
    // }
}
