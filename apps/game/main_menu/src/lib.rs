mod plugin;
pub use plugin::MainMenuPlugin;

pub mod ui;
mod resources;
mod systems;

cfg_if::cfg_if! {
    if #[cfg(not(feature = "no_odst"))] {
        compile_error!("Requires 'no_odst' feature.");
    }
}