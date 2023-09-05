use std::collections::{BTreeMap, HashSet};

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

#[derive(Resource)]
pub struct FileManager {
    pub project_root_entity: Entity,
    pub changelist: BTreeMap<FileEntryKey, Entity>,
    file_entities: HashSet<Entity>,
}

impl FileManager {
    pub fn new(project_root_entity: Entity) -> Self {
        Self {
            project_root_entity,
            changelist: BTreeMap::new(),
            file_entities: HashSet::new(),
        }
    }

    pub fn on_file_create(&mut self, file_entity: &Entity) {
        self.file_entities.insert(*file_entity);
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
}
