use std::collections::{BTreeMap, HashMap};

use bevy_ecs::{
    prelude::{Entity, Resource},
    system::Query,
};
use bevy_log::info;

use naia_bevy_client::Client;

use render_api::components::Visibility;
use render_egui::egui::epaint::ahash::HashSet;

use vortex_proto::{
    components::{FileExtension, FileSystemChild, FileSystemEntry},
    resources::FileEntryKey,
};

use crate::app::{
    components::OwnedByFileLocal,
    resources::{
        camera_manager::CameraManager, canvas::Canvas, edge_manager::EdgeManager,
        input_manager::InputManager, tab_manager::TabManager,
        vertex_manager::VertexManager,
    },
};

struct ChangelistData {
    changelist_entity: Entity,
}

impl ChangelistData {
    fn new(changelist_entity: Entity) -> Self {
        Self { changelist_entity }
    }
}

struct FileData {
    file_type: FileExtension,
    changelist_entity: Option<Entity>,
    changelist_children: HashSet<Entity>,
    // use for, e.g. a skel file required by anim file. anim is the dependent here.
    file_dependents: HashSet<Entity>,
    // use for, e.g. a skel file required by anim file. skel is the dependency here.
    file_dependencies: HashSet<Entity>,
}

impl FileData {
    fn new(file_type: FileExtension) -> Self {
        Self {
            file_type,
            changelist_entity: None,
            changelist_children: HashSet::default(),
            file_dependents: HashSet::default(),
            file_dependencies: HashSet::default(),
        }
    }
}

#[derive(Resource)]
pub struct FileManager {
    pub project_root_entity: Entity,
    changelist: BTreeMap<FileEntryKey, ChangelistData>,
    file_entities: HashMap<Entity, FileData>,
}

impl FileManager {
    pub fn new(project_root_entity: Entity) -> Self {
        Self {
            project_root_entity,
            changelist: BTreeMap::new(),
            file_entities: HashMap::new(),
        }
    }

    pub fn on_file_create(&mut self, file_entity: &Entity, file_type: FileExtension) {
        self.file_entities.insert(*file_entity, FileData::new(file_type));
    }

    pub fn on_file_delete(
        &mut self,
        client: &mut Client,
        canvas: &mut Canvas,
        camera_manager: &mut CameraManager,
        input_manager: &mut InputManager,
        vertex_manager: &mut VertexManager,
        edge_manager: &mut EdgeManager,
        tab_manager: &mut TabManager,
        visibility_q: &mut Query<(&mut Visibility, &OwnedByFileLocal)>,
        file_entity: &Entity,
    ) {
        self.file_entities.remove(file_entity);

        if tab_manager.file_has_tab(file_entity) {
            tab_manager.close_tab(
                client,
                canvas,
                camera_manager,
                input_manager,
                vertex_manager,
                edge_manager,
                visibility_q,
                file_entity,
            );
        }
    }

    pub fn get_file_type(&self, file_entity: &Entity) -> FileExtension {
        let file_data = self.file_entities.get(file_entity).unwrap();
        file_data.file_type
    }

    pub fn insert_changelist_entry(
        &mut self,
        file_entry_key: FileEntryKey,
        file_entity_opt: Option<Entity>,
        parent_entity_opt: Option<Entity>,
        cl_entity: Entity,
    ) {
        self.changelist
            .insert(file_entry_key, ChangelistData::new(cl_entity));

        if let Some(file_entity) = file_entity_opt {
            let Some(file_data) = self.file_entities.get_mut(&file_entity) else {
                panic!("file_entity {:?} not found in file_entities", file_entity);
            };
            file_data.changelist_entity = Some(cl_entity);
        }

        if let Some(parent_entity) = parent_entity_opt {
            let Some(parent_data) = self.file_entities.get_mut(&parent_entity) else {
                panic!("parent_entity {:?} not found in file_entities", parent_entity);
            };
            parent_data.changelist_children.insert(cl_entity);
        }
    }

    pub fn remove_changelist_entry(&mut self, file_entry_key: &FileEntryKey) {
        self.changelist.remove(file_entry_key);
    }

    pub fn changelist_entities(&self) -> Vec<Entity> {
        let mut output = Vec::new();
        for (_, changelist_data) in self.changelist.iter() {
            output.push(changelist_data.changelist_entity);
        }
        output
    }

    pub fn get_file_changelist_entity(&self, file_entity: &Entity) -> Option<Entity> {
        let file_data = self.file_entities.get(file_entity)?;
        let changelist_entity = file_data.changelist_entity?;
        Some(changelist_entity)
    }

    pub fn get_file_changelist_children(&self, file_entity: &Entity) -> Option<&HashSet<Entity>> {
        let file_data = self.file_entities.get(file_entity)?;
        Some(&file_data.changelist_children)
    }

    pub(crate) fn file_has_dependency(&self, file_entity: &Entity, file_type: FileExtension) -> bool {
        let file_data = self.file_entities.get(file_entity).unwrap();
        for dependency_file_entity in file_data.file_dependencies.iter() {
            let dependency_file_data = self.file_entities.get(dependency_file_entity).unwrap();
            if dependency_file_data.file_type == file_type {
                return true;
            }
        }
        false
    }

    pub(crate) fn file_get_dependency(&self, file_entity: &Entity, file_ext: FileExtension) -> Option<Entity> {
        let file_data = self.file_entities.get(&file_entity).unwrap();
        for dependency_file_entity in file_data.file_dependencies.iter() {
            let dependency_file_data = self.file_entities.get(dependency_file_entity).unwrap();
            if dependency_file_data.file_type == file_ext {
                return Some(*dependency_file_entity);
            }
        }
        None
    }

    pub fn file_add_dependency(&mut self, file_entity: &Entity, dependency_file_entity: &Entity) {
        info!("file_add_dependency({:?}, {:?})", file_entity, dependency_file_entity);

        // add dependency to dependent
        {
            let dependent_file_data = self.file_entities.get_mut(file_entity).unwrap();

            if dependent_file_data.file_dependencies.contains(dependency_file_entity) {
                panic!("file {:?} already has dependency {:?}", file_entity, dependency_file_entity);
            }

            dependent_file_data.file_dependencies.insert(*dependency_file_entity);
        }

        // add dependent to dependency
        {
            let dependency_file_data = self.file_entities.get_mut(dependency_file_entity).unwrap();

            if dependency_file_data.file_dependents.contains(file_entity) {
                panic!("file {:?} already has dependent {:?}", dependency_file_entity, file_entity);
            }

            dependency_file_data.file_dependents.insert(*file_entity);
        }
    }

    pub fn file_remove_dependency(&mut self, file_entity: &Entity, dependency_file_entity: &Entity) {
        // remove dependency from dependent
        {
            let dependent_file_data = self.file_entities.get_mut(file_entity).unwrap();

            if !dependent_file_data.file_dependencies.contains(dependency_file_entity) {
                panic!("file {:?} does not have dependency {:?}", file_entity, dependency_file_entity);
            }

            dependent_file_data.file_dependencies.remove(dependency_file_entity);
        }
        // remove dependent from dependency
        {
            let dependency_file_data = self.file_entities.get_mut(dependency_file_entity).unwrap();

            if !dependency_file_data.file_dependents.contains(file_entity) {
                panic!("file {:?} does not have dependent {:?}", dependency_file_entity, file_entity);
            }

            dependency_file_data.file_dependents.remove(file_entity);
        }
    }
}

pub fn get_full_path(
    client: &Client,
    fs_q: &Query<(&FileSystemEntry, Option<&FileSystemChild>)>,
    file_entity: Entity,
) -> String {
    let mut path = String::new();

    let (_, parent) = fs_q.get(file_entity).unwrap();
    if let Some(parent_entity) = parent {
        let mut current_entity = parent_entity.parent_id.get(client).unwrap();

        loop {
            let (entry, parent) = fs_q.get(current_entity).unwrap();
            let entry_name = (*(entry.name)).clone();
            path.insert_str(0, &entry_name);
            if let Some(parent_entity) = parent {
                current_entity = parent_entity.parent_id.get(client).unwrap();
                path.insert_str(0, "/");
            } else {
                break;
            }
        }
    }

    path
}
