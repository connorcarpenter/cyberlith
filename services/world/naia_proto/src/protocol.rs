use std::time::Duration;

use naia_bevy_shared::{LinkConditionerConfig, Protocol};

use crate::{channels::ChannelsPlugin, components::ComponentsPlugin, messages::MessagesPlugin};

// Protocol Build
pub fn protocol() -> Protocol {
    let mut builder = Protocol::builder();

        // Config
    builder
        .rtc_endpoint("world_rtc".to_string())
        .tick_interval(Duration::from_millis(40))
        .enable_client_authoritative_entities()
        // Channels
        .add_plugin(ChannelsPlugin)
        // Messages
        .add_plugin(MessagesPlugin)
        // Components
        .add_plugin(ComponentsPlugin);

    #[cfg(feature = "local")]
    builder.link_condition(LinkConditionerConfig::good_condition());

    // Build Protocol
    builder.build()
}
