use naia_bevy_shared::{Protocol, ProtocolPlugin};

mod auth;
pub use auth::Auth;

mod world_connect;
pub use world_connect::WorldConnectToken;

mod load_asset_request;
pub use load_asset_request::{LoadAssetRequest, LoadAssetResponse, LoadAssetResponseValue};

mod load_asset_with_data;
pub use load_asset_with_data::LoadAssetWithData;

// Plugin
pub struct MessagesPlugin;

impl ProtocolPlugin for MessagesPlugin {
    fn build(&self, protocol: &mut Protocol) {
        protocol
            .add_message::<Auth>()
            .add_message::<WorldConnectToken>()
            .add_message::<LoadAssetWithData>()
            .add_request::<LoadAssetRequest>();
    }
}
