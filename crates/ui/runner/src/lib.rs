mod ui_manager;
mod plugin;
mod runtime;

pub use ui_manager::{UiManager, Blinkiness};
pub use plugin::UiPlugin;
pub use runtime::UiRuntime;

pub mod config {
    pub use ui_runner_config::*;
}
pub mod state {
    pub use ui_state::*;
}
pub mod input {
    pub use ui_input::*;
}