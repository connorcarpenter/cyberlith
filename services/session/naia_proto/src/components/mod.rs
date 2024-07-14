mod message_public;
mod message_global;
mod message_local;

mod lobby_public;
mod lobby_global;
mod lobby_local;

mod user_public;
mod user_self;
mod user_lobby_owner;
mod user_lobby_peer;

pub use message_public::MessagePublic;
pub use message_global::MessageGlobal;
pub use message_local::MessageLocal;

pub use lobby_global::LobbyGlobal;
pub use lobby_local::LobbyLocal;
pub use lobby_public::LobbyPublic;

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
            .add_component::<MessageGlobal>()
            .add_component::<MessageLocal>()

            .add_component::<LobbyPublic>()
            .add_component::<LobbyGlobal>()
            .add_component::<LobbyLocal>()

            .add_component::<UserPublic>()
            .add_component::<UserSelf>()
            .add_component::<UserLobbyPeer>()
            .add_component::<UserLobbyOwner>();
    }
}
