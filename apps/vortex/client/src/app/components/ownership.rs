use bevy_ecs::{component::Component, entity::Entity};

// FileOwnership
#[derive(Component)]
pub struct OwnedByFileLocal {
    pub file_entity: Entity,
}

impl OwnedByFileLocal {
    pub fn new(file_entity: Entity) -> Self {
        Self { file_entity }
    }
}