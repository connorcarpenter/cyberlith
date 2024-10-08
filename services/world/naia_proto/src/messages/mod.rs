mod auth;
mod entity_assignment;
mod key_command;

pub use auth::Auth;
pub use entity_assignment::EntityAssignment;
pub use key_command::KeyCommand;

use naia_bevy_shared::{Protocol, ProtocolPlugin};

pub use naia_bevy_shared::{FakeEntityConverter, Message};

// Plugin
pub(crate) struct MessagesPlugin;

impl ProtocolPlugin for MessagesPlugin {
    fn build(&self, protocol: &mut Protocol) {
        protocol
            .add_message::<Auth>()
            .add_message::<EntityAssignment>()
            .add_message::<KeyCommand>();
    }
}
