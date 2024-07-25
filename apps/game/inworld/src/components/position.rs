use bevy_ecs::prelude::Component;

#[derive(Component, Clone)]
pub struct BufferedNextTilePosition {
    pub x: i16,
    pub y: i16,
}

impl BufferedNextTilePosition {
    pub fn new(x: i16, y: i16) -> Self {
        Self { x, y }
    }
}