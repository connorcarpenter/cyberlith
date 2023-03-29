use bevy_ecs::component::Component;

pub struct RenderLayers {}

impl RenderLayers {
    pub const TOTAL_LAYERS: usize = 32;
}

#[derive(Component)]
pub struct RenderLayer(pub usize);
