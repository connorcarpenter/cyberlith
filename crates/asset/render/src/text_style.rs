use bevy_ecs::component::Component;

#[derive(Component)]
pub struct TextStyle {
    // height of characters in pixels
    pub size: f32,
}

impl TextStyle {
    pub fn new(size: f32) -> Self {
        Self {
            size
        }
    }
}