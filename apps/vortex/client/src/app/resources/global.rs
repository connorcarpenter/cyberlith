use std::collections::BTreeMap;

use bevy_ecs::prelude::{Entity, Resource};

use vortex_proto::components::EntryKind;

#[derive(Resource)]
pub struct Global {
    pub project_root_entity: Entity,
    pub changelist: BTreeMap<(EntryKind, String), Entity>,
}

impl Global {
    pub fn new(project_root_entity: Entity) -> Self {
        Self {
            project_root_entity,
            changelist: BTreeMap::new(),
        }
    }
}