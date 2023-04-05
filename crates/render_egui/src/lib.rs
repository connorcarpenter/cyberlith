pub mod egui {
    pub use egui::*;
}

mod base_set;
mod gui;
mod input;
mod plugin;
pub mod systems;

pub use base_set::*;
pub use gui::*;
pub use input::*;
pub use plugin::*;
