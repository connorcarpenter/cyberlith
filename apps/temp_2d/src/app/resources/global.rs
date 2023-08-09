use bevy_ecs::{entity::Entity, system::Resource};

#[derive(Resource)]
pub struct Global {
    pub solid_circle: Option<Entity>,
    pub hollow_circle: Option<Entity>,
}

impl Global {
    pub fn new() -> Self {
        Self {
            solid_circle: None,
            hollow_circle: None,
        }
    }
}
