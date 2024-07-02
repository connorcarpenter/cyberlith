use bevy_ecs::{prelude::Component, entity::Entity};

use naia_bevy_shared::{EntityAndGlobalEntityConverter, EntityProperty, Property, Replicate};

use social_server_types::{GlobalChatMessageId, Timestamp};

#[derive(Component, Replicate)]
pub struct GlobalChatMessage {
    pub id: Property<GlobalChatMessageId>,
    pub timestamp: Property<Timestamp>,
    pub user_entity: EntityProperty,
    pub message: Property<String>,
}

impl GlobalChatMessage {
    pub fn new(
        converter: &dyn EntityAndGlobalEntityConverter<Entity>,
        id: GlobalChatMessageId,
        timestamp: Timestamp,
        user_entity: Entity,
        message: &str,
    ) -> Self {
        let mut me = Self::new_complete(id, timestamp, message.to_string());
        me.user_entity.set(converter, &user_entity);
        me
    }
}
