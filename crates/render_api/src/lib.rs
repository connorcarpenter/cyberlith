#[macro_use]
extern crate cfg_if;

mod assets;
pub use assets::{shape, Assets, ClearColorConfig, Color, Handle, Image, Mesh, StandardMaterial};

mod components;
pub use components::{
    Camera, ClearOperation, PointLight, PointLightBundle, RenderLayer, RenderLayers,
    RenderObjectBundle, RenderTarget, Transform,
};

mod resources;
pub use resources::Window;

mod plugin;
pub use plugin::RenderApiPlugin;

pub mod math;
