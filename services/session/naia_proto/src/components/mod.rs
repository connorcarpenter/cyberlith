mod global_chat_message;
mod public_user_info;
mod match_lobby;

pub use global_chat_message::GlobalChatMessage;
pub use public_user_info::PublicUserInfo;
pub use match_lobby::MatchLobby;

use naia_bevy_shared::{Protocol, ProtocolPlugin};

// Plugin
pub struct ComponentsPlugin;

impl ProtocolPlugin for ComponentsPlugin {
    fn build(&self, protocol: &mut Protocol) {
        protocol
            .add_component::<GlobalChatMessage>()
            .add_component::<PublicUserInfo>()
            .add_component::<MatchLobby>();
    }
}
