use naia_bevy_shared::{
    Protocol, ProtocolPlugin,
};

// Plugin
pub struct ChannelsPlugin;

impl ProtocolPlugin for ChannelsPlugin {
    fn build(&self, _protocol: &mut Protocol) {
        // nothing here yet
    }
}
