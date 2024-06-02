mod ui_manager;
pub use ui_manager::{Blinkiness, UiManager};

mod plugin;
pub use plugin::UiPlugin;

mod runtime;
pub use runtime::UiRuntime;

mod handle;
mod state_globals;
mod systems;

mod panel_mut;
pub use panel_mut::PanelMut;

pub use handle::UiHandle;

pub mod config {
    pub use ui_runner_config::*;
}
pub mod state {
    pub use ui_state::*;
}
pub mod input {
    pub use ui_input::*;
}
