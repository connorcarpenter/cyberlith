use bevy_ecs::entity::Entity;

use naia_bevy_shared::{EntityAndGlobalEntityConverter, EntityProperty, Message};

use crate::types::TabId;

#[derive(Message)]
pub struct TabOpenMessage {
    pub file_entity: EntityProperty,
    pub tab_id: TabId,
}

impl TabOpenMessage {
    pub fn new(
        converter: &dyn EntityAndGlobalEntityConverter<Entity>,
        tab_id: TabId,
        entity: &Entity,
    ) -> Self {
        let mut new = Self {
            file_entity: EntityProperty::new(),
            tab_id,
        };
        new.file_entity.set(converter, entity);
        new
    }
}

#[derive(Message)]
pub struct TabCloseMessage {
    pub tab_id: TabId,
}

impl TabCloseMessage {
    pub fn new(tab_id: TabId) -> Self {
        Self { tab_id }
    }
}
