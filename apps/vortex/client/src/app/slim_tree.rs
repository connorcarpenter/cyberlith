use bevy_ecs::entity::Entity;
use vortex_proto::components::EntryKind;

pub struct SlimTree {
    pub entity: Entity,
    pub name: String,
    pub kind: EntryKind,
    pub children: Option<Vec<SlimTree>>,
}

impl SlimTree {
    pub fn new(entity: Entity, name: String, kind: EntryKind) -> Self {
        Self {
            entity,
            name,
            kind,
            children: None,
        }
    }
}