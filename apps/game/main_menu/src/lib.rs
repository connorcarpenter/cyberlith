mod plugin;
pub use plugin::MainMenuPlugin;

mod resources;
mod systems;
pub mod ui;

cfg_if::cfg_if! {
    if #[cfg(not(feature = "no_odst"))] {
        compile_error!("Requires 'no_odst' feature.");
    }
}
