use bevy_ecs::prelude::Component;

#[derive(Component)]
pub struct ModelTransformControl;

impl ModelTransformControl {
    pub const RADIUS: f32 = 1.5;
}
