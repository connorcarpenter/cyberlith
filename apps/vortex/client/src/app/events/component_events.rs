use bevy_ecs::{entity::Entity, event::Event};

use naia_bevy_client::Replicate;

#[derive(Event)]
pub struct InsertComponentEvent<T: Replicate> {
    pub entity: Entity,
    phantom_t: std::marker::PhantomData<T>,
}

impl<T: Replicate> InsertComponentEvent<T> {
    pub fn new(entity: Entity) -> Self {
        Self {
            entity,
            phantom_t: std::marker::PhantomData,
        }
    }
}

#[derive(Event)]
pub struct RemoveComponentEvent<T: Replicate> {
    pub entity: Entity,
    pub component: T,
}

impl<T: Replicate> RemoveComponentEvent<T> {
    pub fn new(entity: Entity, component: T) -> Self {
        Self { entity, component }
    }
}
