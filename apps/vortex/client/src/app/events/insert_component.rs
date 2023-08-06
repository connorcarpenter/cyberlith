use bevy_ecs::entity::Entity;

use naia_bevy_client::Replicate;

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
