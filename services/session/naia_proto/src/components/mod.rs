mod user;

mod chat_message;
mod chat_message_global;
mod chat_message_local;

mod lobby;
mod lobby_member;

mod selfhood;
mod selfhood_user;

//

pub use user::User;

pub use chat_message::ChatMessage;
pub use chat_message_global::ChatMessageGlobal;
pub use chat_message_local::ChatMessageLocal;

pub use lobby::Lobby;
pub use lobby_member::LobbyMember;

pub use selfhood::Selfhood;
pub use selfhood_user::SelfhoodUser;

//

use naia_bevy_shared::{Protocol, ProtocolPlugin};

// Plugin
pub struct ComponentsPlugin;

impl ProtocolPlugin for ComponentsPlugin {
    fn build(&self, protocol: &mut Protocol) {
        protocol
            .add_component::<User>()
            .add_component::<ChatMessage>()
            .add_component::<ChatMessageGlobal>()
            .add_component::<ChatMessageLocal>()
            .add_component::<Lobby>()
            .add_component::<LobbyMember>()
            .add_component::<Selfhood>()
            .add_component::<SelfhoodUser>();
    }
}
