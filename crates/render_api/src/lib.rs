#[macro_use]
extern crate cfg_if;

mod assets;
mod base_set;
mod components;
pub mod math;
mod plugin;
mod resources;

pub use assets::{shape, Assets, ClearColorConfig, Color, Handle, Image, Material, Mesh};
pub use base_set::RenderSet;
pub use components::{
    Camera, ClearOperation, PointLight, PointLightBundle, RenderLayer, RenderLayers,
    RenderObjectBundle, RenderTarget, Transform,
};
pub use plugin::RenderApiPlugin;
pub use resources::Window;
