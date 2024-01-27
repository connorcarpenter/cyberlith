use bevy_ecs::{entity::Entity, event::Event};

use naia_bevy_server::{Replicate, UserKey};

#[derive(Event)]
pub struct InsertComponentEvent<T: Replicate> {
    pub user_key: UserKey,
    pub entity: Entity,
    phantom_t: std::marker::PhantomData<T>,
}

impl<T: Replicate> InsertComponentEvent<T> {
    pub fn new(user_key: UserKey, entity: Entity) -> Self {
        Self {
            user_key,
            entity,
            phantom_t: std::marker::PhantomData,
        }
    }
}
