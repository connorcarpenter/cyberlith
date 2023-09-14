use naia_bevy_shared::{Protocol, ProtocolPlugin};

mod auth;
mod changelist;
mod tabs;
mod bind;

pub use auth::*;
pub use changelist::*;
pub use tabs::*;
pub use bind::*;

// Plugin
pub struct MessagesPlugin;

impl ProtocolPlugin for MessagesPlugin {
    fn build(&self, protocol: &mut Protocol) {
        protocol
            .add_message::<Auth>()
            .add_message::<ChangelistMessage>()
            .add_message::<TabActionMessage>()
            .add_message::<TabOpenMessage>()
            .add_message::<FileBindMessage>();
    }
}
