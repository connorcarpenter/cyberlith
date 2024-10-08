use bevy_ecs::entity::Entity;

use editor_proto::components::EntryKind;

#[derive(Clone)]
pub struct FileTree {
    pub entity: Entity,
    pub name: String,
    pub kind: EntryKind,
    pub children: Option<Vec<FileTree>>,
}

impl FileTree {
    pub fn new(entity: Entity, name: String, kind: EntryKind) -> Self {
        Self {
            entity,
            name,
            kind,
            children: None,
        }
    }
}
