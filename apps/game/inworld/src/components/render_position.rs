
use bevy_ecs::component::Component;

#[derive(Component, Clone)]
pub struct RenderPosition {

}

impl RenderPosition {
    pub fn new() -> Self {
        Self {

        }
    }
}
