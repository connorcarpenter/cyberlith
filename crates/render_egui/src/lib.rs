pub mod egui {
    pub use egui::*;
}

mod base_set;
mod draw;
mod gui;
mod input;
mod plugin;
pub mod resources;
pub mod systems;

pub use base_set::*;
pub use draw::*;
pub use gui::*;
pub use input::*;
pub use plugin::*;
