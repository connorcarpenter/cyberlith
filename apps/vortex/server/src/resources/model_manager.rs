use bevy_ecs::{entity::Entity, system::Resource};

#[derive(Resource)]
pub struct ModelManager {}

impl Default for ModelManager {
    fn default() -> Self {
        Self {}
    }
}

impl ModelManager {
    pub fn has_model_transform(&self, model_entity: &Entity) -> bool {
        todo!()
    }

    pub fn on_create_model_transform(&mut self, model_entity: &Entity) {
        todo!()
    }

    pub fn on_despawn_model_transform(&mut self, model_entity: &Entity) {
        self.deregister_model_transform(model_entity);
    }

    pub fn deregister_model_transform(&mut self, model_entity: &Entity) {}
}
