#[macro_use]
extern crate cfg_if;

mod plugin;
pub use plugin::EnginePlugin;

mod renderer;

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
pub mod bevy_http_client {
    pub use bevy_http_client::*;
}
pub mod naia {
    pub use naia_bevy_client::*;
}
pub mod orchestrator {
    pub use orchestrator_proto::*;
}
