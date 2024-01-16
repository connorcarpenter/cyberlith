use naia_bevy_shared::{
    Channel, ChannelDirection, ChannelMode, Protocol, ProtocolPlugin, ReliableSettings,
};

// Plugin
pub struct ChannelsPlugin;

impl ProtocolPlugin for ChannelsPlugin {
    fn build(&self, protocol: &mut Protocol) {
        protocol
            .add_channel::<TabActionChannel>(
                ChannelDirection::ClientToServer,
                ChannelMode::OrderedReliable(ReliableSettings::default()),
            )
            .add_channel::<FileActionChannel>(
                ChannelDirection::Bidirectional,
                ChannelMode::OrderedReliable(ReliableSettings::default()),
            );
    }
}

// FileActionChannel
#[derive(Channel)]
pub struct FileActionChannel;

// TabActionChannel
#[derive(Channel)]
pub struct TabActionChannel;
