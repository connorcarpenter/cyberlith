use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature = "editor")] {
        pub use three_d::egui;
        #[path = "egui.rs"]
        mod egui_internal;
        pub use egui_internal::{EguiContext, EguiUserTextures, EguiPlugin, EguiContexts};
    }
}

pub use three_d::{ClearState, Viewport, Camera as InnerCamera, vec3, Gm, Object};

mod assets;
pub use assets::{shape, Assets, ClearColorConfig, Color, Handle, Image, Mesh, StandardMaterial};

mod components;
pub use components::{
    Camera, PointLight, PointLightBundle,
    RenderObjectBundle, RenderTarget, Transform, RenderLayers, RenderLayer,
};

mod systems;

mod resources;
pub use resources::Window;

mod plugin;
pub use plugin::RenderPlugin;

mod runner;

pub mod math;
