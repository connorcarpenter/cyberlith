use std::time::Duration;

use naia_bevy_shared::{LinkConditionerConfig, Protocol};

use crate::{channels::ChannelsPlugin, messages::MessagesPlugin};

// Protocol Build
pub fn protocol() -> Protocol {
    let mut builder = Protocol::builder();

    // Config
    builder
        .rtc_endpoint("session_rtc".to_string())
        .tick_interval(Duration::from_millis(40))
        .enable_client_authoritative_entities()
        // Channels
        .add_plugin(ChannelsPlugin)
        // Messages
        .add_plugin(MessagesPlugin);

    #[cfg(feature = "local")]
    builder.link_condition(LinkConditionerConfig::good_condition());

    // Build Protocol
    builder.build()
}
