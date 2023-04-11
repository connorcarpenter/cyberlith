pub mod widgets;
pub mod workspaces;

mod center_panel;
mod left_panel;
mod login_modal;
mod main;
mod right_panel;
mod state;
mod top_bar;

pub use center_panel::*;
pub use left_panel::*;
pub use login_modal::login_modal;
pub use main::*;
pub use right_panel::*;
pub use state::*;
pub use top_bar::*;
