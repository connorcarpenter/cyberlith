use naia_bevy_shared::{Protocol, ProtocolPlugin};

mod global_chat_send_message;
mod match_lobby_create;
mod match_lobby_join;
mod match_lobby_leave;
mod match_lobby_send_message;
mod match_lobby_game_start;

use global_chat_send_message::GlobalChatSendMessage;
use match_lobby_create::MatchLobbyCreate;
use match_lobby_join::MatchLobbyJoin;
use match_lobby_leave::MatchLobbyLeave;
use match_lobby_send_message::MatchLobbySendMessage;
use match_lobby_game_start::MatchLobbyGameStart;

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