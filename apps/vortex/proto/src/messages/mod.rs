use naia_bevy_shared::{Protocol, ProtocolPlugin};

mod auth;
mod changelist;

pub use auth::Auth;
pub use changelist::{ChangelistMessage, ChangelistAction};

// Plugin
pub struct MessagesPlugin;

impl ProtocolPlugin for MessagesPlugin {
    fn build(&self, protocol: &mut Protocol) {
        protocol
            .add_message::<Auth>()
            .add_message::<ChangelistMessage>();
    }
}
