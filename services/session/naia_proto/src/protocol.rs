use std::time::Duration;

use naia_bevy_shared::Protocol;

use crate::{channels::ChannelsPlugin, messages::MessagesPlugin};

// Protocol Build
pub fn protocol() -> Protocol {
    let mut builder = Protocol::builder();

    // Config
    builder
        .rtc_endpoint("api/session_connect".to_string())
        .tick_interval(Duration::from_millis(40))
        .enable_client_authoritative_entities()
        // Channels
        .add_plugin(ChannelsPlugin)
        // Messages
        .add_plugin(MessagesPlugin);

    cfg_if::cfg_if! {
        if #[cfg(feature = "local")]{
            use naia_bevy_shared::LinkConditionerConfig;

            builder.link_condition(LinkConditionerConfig::good_condition());
        }
        else {}
    }

    // Build Protocol
    builder.build()
}
