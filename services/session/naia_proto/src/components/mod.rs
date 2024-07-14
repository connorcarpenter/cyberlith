mod chat_message;
mod chat_message_global;
mod chat_message_local;

mod lobby;

mod user;
mod user_lobby_owner;
mod user_lobby_peer;
mod user_self;

pub use chat_message::ChatMessage;
pub use chat_message_global::ChatMessageGlobal;
pub use chat_message_local::ChatMessageLocal;

pub use lobby::Lobby;

pub use user::User;
pub use user_lobby_owner::UserLobbyOwner;
pub use user_lobby_peer::UserLobbyPeer;
pub use user_self::UserSelf;

use naia_bevy_shared::{Protocol, ProtocolPlugin};

// Plugin
pub struct ComponentsPlugin;

impl ProtocolPlugin for ComponentsPlugin {
    fn build(&self, protocol: &mut Protocol) {
        protocol
            .add_component::<ChatMessage>()
            .add_component::<ChatMessageGlobal>()
            .add_component::<ChatMessageLocal>()
            .add_component::<Lobby>()
            .add_component::<User>()
            .add_component::<UserSelf>()
            .add_component::<UserLobbyPeer>()
            .add_component::<UserLobbyOwner>();
    }
}
