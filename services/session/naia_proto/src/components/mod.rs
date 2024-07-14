mod lobby_global;
mod lobby_public;
mod message_public;
mod user_lobby_owner;
mod user_lobby_peer;
mod user_public;
mod user_self;

pub use lobby_global::LobbyGlobal;
pub use lobby_public::LobbyPublic;
pub use message_public::MessagePublic;
pub use user_lobby_owner::UserLobbyOwner;
pub use user_lobby_peer::UserLobbyPeer;
pub use user_public::UserPublic;
pub use user_self::UserSelf;

use naia_bevy_shared::{Protocol, ProtocolPlugin};

// Plugin
pub struct ComponentsPlugin;

impl ProtocolPlugin for ComponentsPlugin {
    fn build(&self, protocol: &mut Protocol) {
        protocol
            .add_component::<MessagePublic>()
            .add_component::<UserPublic>()
            .add_component::<UserSelf>()
            .add_component::<UserLobbyPeer>()
            .add_component::<UserLobbyOwner>()
            .add_component::<LobbyPublic>()
            .add_component::<LobbyGlobal>();
    }
}
