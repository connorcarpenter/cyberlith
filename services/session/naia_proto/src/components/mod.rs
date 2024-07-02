mod global_chat_message;
mod present_user_info;

pub use global_chat_message::GlobalChatMessage;
pub use present_user_info::PresentUserInfo;

use naia_bevy_shared::{Protocol, ProtocolPlugin};

// Plugin
pub struct ComponentsPlugin;

impl ProtocolPlugin for ComponentsPlugin {
    fn build(&self, protocol: &mut Protocol) {
        protocol
            .add_component::<GlobalChatMessage>()
            .add_component::<PresentUserInfo>();
    }
}
