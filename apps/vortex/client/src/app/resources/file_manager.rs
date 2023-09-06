use std::collections::{BTreeMap, HashMap};

use bevy_ecs::{
    prelude::{Entity, Resource},
    system::Query,
};

use naia_bevy_client::Client;

use render_api::components::Visibility;
use vortex_proto::resources::FileEntryKey;

use crate::app::{
    components::OwnedByFileLocal,
    resources::{
        camera_manager::CameraManager, canvas::Canvas, shape_manager::ShapeManager,
        tab_manager::TabManager, toolbar::Toolbar,
    },
};

struct ChangelistData {
    changelist_entity: Entity,
}

impl ChangelistData {
    fn new(changelist_entity: Entity) -> Self {
        Self {
            changelist_entity,
        }
    }
}

struct FileData {
    changelist_entity: Option<Entity>,
}

impl FileData {
    fn new() -> Self {
        Self {
            changelist_entity: None,
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

    pub fn on_file_create(&mut self, file_entity: &Entity) {
        self.file_entities.insert(*file_entity, FileData::new());
    }

    pub fn on_file_delete(
        &mut self,
        client: &mut Client,
        canvas: &mut Canvas,
        camera_manager: &mut CameraManager,
        shape_manager: &mut ShapeManager,
        tab_manager: &mut TabManager,
        toolbar: &mut Toolbar,
        visibility_q: &mut Query<(&mut Visibility, &OwnedByFileLocal)>,
        file_entity: &Entity,
    ) {
        self.file_entities.remove(file_entity);

        if tab_manager.file_has_tab(file_entity) {
            tab_manager.close_tab(
                client,
                canvas,
                camera_manager,
                shape_manager,
                toolbar,
                visibility_q,
                file_entity,
            );
        }
    }

    pub fn insert_changelist_entry(&mut self, file_entry_key: FileEntryKey, file_entity_opt: Option<Entity>, cl_entity: Entity) {
        self.changelist.insert(file_entry_key, ChangelistData::new(cl_entity));

        if let Some(file_entity) = file_entity_opt {
            let Some(file_data) = self.file_entities.get_mut(&file_entity) else {
                panic!("file_entity not found in file_entities");
            };
            file_data.changelist_entity = Some(cl_entity);
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
}
