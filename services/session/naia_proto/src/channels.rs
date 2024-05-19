use naia_bevy_shared::{
    Channel, ChannelDirection, ChannelMode, Protocol, ProtocolPlugin, ReliableSettings,
};

#[derive(Channel)]
pub struct PrimaryChannel;

#[derive(Channel)]
pub struct AssetRequestsChannel;

#[derive(Channel)]
pub struct ClientActionsChannel;

// Plugin
pub struct ChannelsPlugin;

impl ProtocolPlugin for ChannelsPlugin {
    fn build(&self, protocol: &mut Protocol) {
        protocol
            .add_channel::<PrimaryChannel>(
                ChannelDirection::ServerToClient,
                ChannelMode::UnorderedReliable(ReliableSettings::default()),
            )
            .add_channel::<AssetRequestsChannel>(
                ChannelDirection::Bidirectional,
                ChannelMode::UnorderedReliable(ReliableSettings::default()),
            )
            .add_channel::<ClientActionsChannel>(
                ChannelDirection::ClientToServer,
                ChannelMode::OrderedReliable(ReliableSettings::default()),
            )
        ;
    }
}
