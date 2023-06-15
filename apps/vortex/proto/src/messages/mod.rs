use naia_bevy_shared::{Protocol, ProtocolPlugin};

pub use auth::Auth;
pub use changelist::{ChangelistAction, ChangelistMessage};
pub use tabs::{TabActionMessage, TabActionMessageType, TabOpenMessage};

mod auth;
mod changelist;
mod tabs;

// Plugin
pub struct MessagesPlugin;

impl ProtocolPlugin for MessagesPlugin {
    fn build(&self, protocol: &mut Protocol) {
        protocol
            .add_message::<Auth>()
            .add_message::<ChangelistMessage>()
            .add_message::<TabActionMessage>()
            .add_message::<TabOpenMessage>();
    }
}
