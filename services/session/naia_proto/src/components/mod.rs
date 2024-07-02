mod global_chat_message;
mod public_user_info;

pub use global_chat_message::GlobalChatMessage;
pub use public_user_info::PublicUserInfo;

use naia_bevy_shared::{Protocol, ProtocolPlugin};

// Plugin
pub struct ComponentsPlugin;

impl ProtocolPlugin for ComponentsPlugin {
    fn build(&self, protocol: &mut Protocol) {
        protocol
            .add_component::<GlobalChatMessage>()
            .add_component::<PublicUserInfo>();
    }
}
