use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature = "editor")] {
        pub use egui;
        #[path = "egui.rs"]
        mod egui_internal;
        pub use egui_internal::{EguiContext, EguiUserTextures};
    }
}

pub mod shape;
pub mod math;
mod assets;
mod image;
mod window;
mod mesh;
mod material;
mod object;
mod light;
mod camera;
mod transform;
mod color;

pub use assets::{Assets, Handle};
pub use image::Image;
pub use window::Window;
pub use mesh::Mesh;
pub use material::StandardMaterial;
pub use object::RenderObjectBundle;
pub use light::{PointLightBundle, PointLight};
pub use camera::{Camera3dBundle, Camera, Camera3d, RenderTarget};
pub use color::{Color, ClearColorConfig};
pub use transform::Transform;