use naia_bevy_shared::{Protocol, ProtocolPlugin};

// exposing these for use in the gateway
pub use naia_bevy_shared::{FakeEntityConverter, Message};

mod auth;
pub use auth::{Auth, AuthInner};

mod world_connect_token;
pub use world_connect_token::WorldConnectToken;

mod load_asset_request;
pub use load_asset_request::{LoadAssetRequest, LoadAssetResponse, LoadAssetResponseValue};

mod load_asset_with_data;
pub use load_asset_with_data::LoadAssetWithData;

mod social;
pub use social::{
    GlobalChatSendMessage, MatchLobbyCreate, MatchLobbyGameStart, MatchLobbyJoin, MatchLobbyLeave,
    MatchLobbySendMessage,
};

// Plugin
pub struct MessagesPlugin;

impl ProtocolPlugin for MessagesPlugin {
    fn build(&self, protocol: &mut Protocol) {
        protocol
            .add_plugin(social::SocialMessagesPlugin)
            .add_message::<Auth>()
            .add_message::<WorldConnectToken>()
            .add_message::<LoadAssetWithData>()
            .add_request::<LoadAssetRequest>();
    }
}
