use bevy_ecs::component::Component;

#[derive(Component, Clone)]
pub struct AnimationState {
    pub(crate) animation_name: String,
    pub(crate) animation_index_ms: f32,
}

impl AnimationState {
    pub fn new() -> Self {
        Self {
            animation_name: "idle".to_string(),
            animation_index_ms: 0.0,
        }
    }
}
