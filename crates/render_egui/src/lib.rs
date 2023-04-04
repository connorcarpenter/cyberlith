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

mod base_set;
mod plugin;

pub use base_set::*;
pub use plugin::*;
