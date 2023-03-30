#[macro_use]
extern crate cfg_if;

mod asset_impls;
mod core;
mod draw;
mod plugin;
mod renderer;
mod runner;
mod sync;
mod window;

pub use plugin::RenderGlowPlugin;

cfg_if! {
    if #[cfg(feature = "editor")] {
        mod egui_gui;
    }
}
