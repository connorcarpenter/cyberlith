use naia_bevy_shared::{Protocol, ProtocolPlugin};

pub use naia_bevy_shared::{FakeEntityConverter, Message};

mod auth;
pub use auth::Auth;

// Plugin
pub(crate) struct MessagesPlugin;

impl ProtocolPlugin for MessagesPlugin {
    fn build(&self, protocol: &mut Protocol) {
        protocol.add_message::<Auth>();
    }
}
