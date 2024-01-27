use naia_bevy_shared::{Channel, ChannelDirection, ChannelMode, Protocol, ProtocolPlugin, ReliableSettings};

#[derive(Channel)]
pub struct PrimaryChannel;

// Plugin
pub struct ChannelsPlugin;

impl ProtocolPlugin for ChannelsPlugin {
    fn build(&self, protocol: &mut Protocol) {
        protocol
            .add_channel::<PrimaryChannel>(
                ChannelDirection::ServerToClient,
                ChannelMode::UnorderedReliable(ReliableSettings::default()),
            );
    }
}
