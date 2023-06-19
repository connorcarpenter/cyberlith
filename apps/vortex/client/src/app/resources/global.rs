use std::collections::BTreeMap;

use bevy_ecs::prelude::{Entity, Resource};

use vortex_proto::resources::FileEntryKey;

#[derive(Resource)]
pub struct Global {
    pub project_root_entity: Entity,
    pub changelist: BTreeMap<FileEntryKey, Entity>,
}

impl Global {
    pub fn new(project_root_entity: Entity) -> Self {
        Self {
            project_root_entity,
            changelist: BTreeMap::new(),
        }
    }
}
