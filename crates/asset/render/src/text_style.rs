use bevy_ecs::component::Component;

#[derive(Component)]
pub struct TextStyle {
    // height of characters in pixels
    pub size: f32,
    // spacing between characters, in pixels
    pub character_buffer: f32,
}

impl TextStyle {
    pub fn new(size: f32, character_buffer: f32) -> Self {
        Self {
            size,
            character_buffer,
        }
    }
}
