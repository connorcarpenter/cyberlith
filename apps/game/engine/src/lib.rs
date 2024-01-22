#[macro_use]
extern crate cfg_if;

mod plugin;
pub use plugin::EnginePlugin;

mod renderer;
mod client_markers;

pub mod asset {
    pub use asset::*;
}
pub mod input {
    pub use input::*;
}
pub mod math {
    pub use math::*;
}
pub mod render {
    pub use render_api::*;
}
pub mod http {
    pub use bevy_http_client::*;
}
pub mod naia {
    pub use naia_bevy_client::*;
}
pub mod orchestrator {
    pub use orchestrator_http_proto::*;
}
pub mod session {
    pub use session_server_naia_proto::*;
    pub use super::client_markers::Session;
}
pub mod world {
    pub use super::client_markers::World;
}