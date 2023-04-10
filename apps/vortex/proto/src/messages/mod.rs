use naia_bevy_shared::{Protocol, ProtocolPlugin};

mod auth;
pub use auth::Auth;

// Plugin
pub struct MessagesPlugin;

impl ProtocolPlugin for MessagesPlugin {
    fn build(&self, protocol: &mut Protocol) {
        protocol.add_message::<Auth>();
    }
}
