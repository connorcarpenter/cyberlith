use bevy_ecs::component::Component;

pub struct RenderLayers;

impl RenderLayers {
    pub const DEFAULT: usize = 0;
    pub const TOTAL_LAYERS: usize = 32;

    pub(crate) const MAX_LAYERS_INTERNAL: usize = RenderLayers::TOTAL_LAYERS + 3;

    pub fn layer(layer: usize) -> RenderLayer {
        if layer >= Self::TOTAL_LAYERS {
            panic!("RenderLayer index out of bounds! Max is {}", Self::TOTAL_LAYERS - 1);
        }
        RenderLayer(layer)
    }
}

#[derive(Component, Clone, Copy, Eq, PartialEq, Debug)]
pub struct RenderLayer(usize);

impl RenderLayer {

    pub const DEFAULT: RenderLayer = RenderLayer(RenderLayers::DEFAULT);
    pub const UI: RenderLayer = RenderLayer(RenderLayers::TOTAL_LAYERS + 1);
    pub const PHYSICS_DEBUG: RenderLayer = RenderLayer(RenderLayers::TOTAL_LAYERS + 2);

    pub fn as_usize(&self) -> usize { self.0 }
}

impl Default for RenderLayer {
    fn default() -> Self {
        RenderLayer(RenderLayers::DEFAULT)
    }
}
