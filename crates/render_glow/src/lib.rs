#[macro_use]
extern crate cfg_if;

mod asset;
mod core;
mod draw;
mod plugin;
mod renderer;
mod runner;
mod window;
mod sync;
mod asset_impls;

pub use plugin::RenderGlowPlugin;

cfg_if! {
    if #[cfg(feature = "editor")] {
        mod egui_gui;
    }
}
