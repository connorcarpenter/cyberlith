#[macro_use]
extern crate cfg_if;

mod assets;
mod base_set;
mod components;
mod resources;
mod plugin;
pub mod math;

pub use assets::{shape, Assets, ClearColorConfig, Color, Handle, Image, Mesh, StandardMaterial};
pub use components::{
    Camera, ClearOperation, PointLight, PointLightBundle, RenderLayer, RenderLayers,
    RenderObjectBundle, RenderTarget, Transform,
};
pub use resources::Window;
pub use plugin::RenderApiPlugin;
pub use base_set::RenderSet;



