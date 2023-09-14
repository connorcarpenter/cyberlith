use bevy_ecs::entity::Entity;

use naia_bevy_shared::{EntityAndGlobalEntityConverter, EntityProperty, Message, Serde};

#[derive(Message)]
pub struct FileBindMessage {
    pub file_entity: EntityProperty,
    pub dependency_entity: EntityProperty,
}

impl FileBindMessage {
    pub fn new(
        converter: &dyn EntityAndGlobalEntityConverter<Entity>,
        file_entity: &Entity,
        dependency_entity: &Entity,
    ) -> Self {
        let mut new = Self {
            file_entity: EntityProperty::new(),
            dependency_entity: EntityProperty::new(),
        };
        new.file_entity.set(converter, file_entity);
        new.dependency_entity.set(converter, dependency_entity);
        new
    }
}