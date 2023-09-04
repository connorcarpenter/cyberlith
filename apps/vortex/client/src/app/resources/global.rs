use std::collections::BTreeMap;

use bevy_ecs::prelude::{Entity, Resource};

use vortex_proto::resources::FileEntryKey;

use crate::app::resources::{action::FileAction, action_stack::ActionStack};

#[derive(Resource)]
pub struct Global {
    pub project_root_entity: Entity,
    pub changelist: BTreeMap<FileEntryKey, Entity>,
    pub action_stack: ActionStack<FileAction>,
}

impl Global {
    pub fn new(project_root_entity: Entity) -> Self {
        Self {
            project_root_entity,
            changelist: BTreeMap::new(),
            action_stack: ActionStack::default(),
        }
    }
}
