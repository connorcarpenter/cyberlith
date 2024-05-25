mod global_chat_message;

use naia_bevy_shared::{Protocol, ProtocolPlugin};

pub use global_chat_message::{GlobalChatMessage};

// Plugin
pub struct ComponentsPlugin;

impl ProtocolPlugin for ComponentsPlugin {
    fn build(&self, protocol: &mut Protocol) {
        protocol
            .add_component::<GlobalChatMessage>();
    }
}