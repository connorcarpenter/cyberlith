use naia_bevy_shared::{Protocol, ProtocolPlugin};

mod auth;
pub use auth::Auth;

mod world_connect;
pub use world_connect::WorldConnectToken;

// Plugin
pub struct MessagesPlugin;

impl ProtocolPlugin for MessagesPlugin {
    fn build(&self, protocol: &mut Protocol) {
        protocol
            .add_message::<Auth>()
            .add_message::<WorldConnectToken>();
    }
}
