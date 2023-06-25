use bevy_ecs::component::Component;

pub struct RenderLayers;

impl RenderLayers {
    pub const DEFAULT: usize = 0;
    pub const TOTAL_LAYERS: usize = 32;

    pub fn layer(layer: usize) -> RenderLayer {
        RenderLayer(layer)
    }
}

#[derive(Component, Clone, Copy, Eq, PartialEq)]
pub struct RenderLayer(pub usize);

impl Default for RenderLayer {
    fn default() -> Self {
        RenderLayer(RenderLayers::DEFAULT)
    }
}
