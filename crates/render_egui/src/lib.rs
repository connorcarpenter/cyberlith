#[macro_use]
extern crate cfg_if;

cfg_if! {
    if #[cfg(feature = "glow_renderer")] {
        pub mod glow {
            pub use egui_glow::*;
        }
    }
}

pub mod egui {
    pub use egui::*;
}

mod resources;
pub use resources::*;

mod plugin;
pub use plugin::*;
