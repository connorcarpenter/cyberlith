use naia_bevy_shared::{Protocol, ProtocolPlugin};

mod global_chat_send_message;
mod match_lobby_create;
mod match_lobby_game_start;
mod match_lobby_join;
mod match_lobby_leave;
mod match_lobby_send_message;

pub use global_chat_send_message::GlobalChatSendMessage;
pub use match_lobby_create::MatchLobbyCreate;
pub use match_lobby_game_start::MatchLobbyGameStart;
pub use match_lobby_join::MatchLobbyJoin;
pub use match_lobby_leave::MatchLobbyLeave;
pub use match_lobby_send_message::MatchLobbySendMessage;

// Plugin
pub struct SocialMessagesPlugin;

impl ProtocolPlugin for SocialMessagesPlugin {
    fn build(&self, protocol: &mut Protocol) {
        protocol
            .add_message::<GlobalChatSendMessage>()
            .add_message::<MatchLobbyCreate>()
            .add_message::<MatchLobbyJoin>()
            .add_message::<MatchLobbyLeave>()
            .add_message::<MatchLobbySendMessage>()
            .add_message::<MatchLobbyGameStart>();
    }
}
