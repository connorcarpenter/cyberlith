use cfg_if::cfg_if;

mod assets;
pub use assets::{shape, Assets, ClearColorConfig, Color, Handle, Image, Mesh, StandardMaterial};

mod components;
pub use components::{
    Camera, PointLight, PointLightBundle,
    RenderObjectBundle, RenderTarget, Transform, RenderLayers, RenderLayer, ClearOperation,
};

mod resources;
pub use resources::Window;

mod plugin;
pub use plugin::RenderApiPlugin;

pub mod math;
