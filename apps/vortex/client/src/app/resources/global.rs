use bevy_ecs::prelude::{Entity, Resource};

#[derive(Resource)]
pub struct Global {
    pub project_root_entity: Entity,
}
