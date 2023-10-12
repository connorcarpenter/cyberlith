use bevy_ecs::component::Component;

#[derive(Component)]
pub struct FaceIndex {
    pub index: usize,
}

impl FaceIndex {
    pub fn new(index: usize) -> Self {
        Self {
            index
        }
    }
}