use bevy_ecs::entity::Entity;
use naia_bevy_shared::{EntityAndGlobalEntityConverter, EntityProperty, Message, Serde};

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

#[derive(Serde, PartialEq, Clone)]
pub enum TabActionMessageType {
    Select,
    Close,
}

#[derive(Message)]
pub struct TabActionMessage {
    pub tab_id: TabId,
    pub action: TabActionMessageType,
}

impl TabActionMessage {
    pub fn new(tab_id: TabId, action: TabActionMessageType) -> Self {
        Self { tab_id, action }
    }
}
