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
    pub use naia_bevy_client::{Timer, transport::webrtc::Socket as WebrtcSocket};
}
pub mod orchestrator {
    pub use orchestrator_http_proto::*;
}
pub mod session {
    use naia_bevy_client::{Client, events::{ConnectEvent, RequestEvents, MessageEvents}};

    use super::client_markers::Session;

    pub type SessionClient<'w> = Client<'w, Session>;
    pub type SessionConnectEvent = ConnectEvent<Session>;
    pub type SessionMessageEvents = MessageEvents<Session>;
    pub type SessionRequestEvents = RequestEvents<Session>;

    pub use session_server_naia_proto::{
        messages::{Auth as SessionAuth, WorldConnectToken, AssetEtagRequest, AssetEtagResponse, AssetDataMessage},
        channels::{PrimaryChannel as SessionPrimaryChannel, RequestChannel as SessionRequestChannel}
    };
}
pub mod world {
    use naia_bevy_client::{Client, events::{ConnectEvent}};

    use super::client_markers::World;

    pub type WorldClient<'w> = Client<'w, World>;
    pub type WorldConnectEvent = ConnectEvent<World>;

    pub use world_server_naia_proto::messages::{Auth as WorldAuth};
}
pub mod config {
    pub use config::*;
}
pub mod storage {
    pub use storage::*;
}
pub use renderer::wait_for_finish;