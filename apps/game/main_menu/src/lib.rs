mod plugin;
pub use plugin::MainMenuPlugin;

pub mod ui;
mod resources;
mod systems;

cfg_if::cfg_if! {
    if #[cfg(feature = "autodriver")] {
        mod autodriver;
    } else {}
}