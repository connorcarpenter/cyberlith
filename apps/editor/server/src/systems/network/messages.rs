use bevy_ecs::{
    event::EventReader,
    system::{Query, ResMut},
};
use logging::info;

use naia_bevy_server::{events::MessageEvents, Server};

use editor_proto::{
    channels::{FileActionChannel, TabActionChannel},
    messages::{ChangelistMessage, TabCloseMessage, TabOpenMessage},
    resources::FileKey,
};

use crate::resources::{ChangelistManager, TabManager};

pub fn message_events(
    server: Server,
    mut event_reader: EventReader<MessageEvents>,
    mut tab_manager: ResMut<TabManager>,
    mut cl_manager: ResMut<ChangelistManager>,
    file_key_q: Query<&FileKey>,
) {
    for events in event_reader.read() {
        // Changelist Message
        for (user_key, message) in events.read::<FileActionChannel, ChangelistMessage>() {
            info!(
                "received ChangelistMessage with action: {:?}",
                message.action
            );

            cl_manager.queue_changelist_message(user_key, message);
        }

        // Tab Open Message
        for (user_key, message) in events.read::<TabActionChannel, TabOpenMessage>() {
            let tab_id = message.tab_id;
            if let Some(file_entity) = message.file_entity.get(&server) {
                tab_manager.queue_open_tab(&file_key_q, &user_key, &tab_id, &file_entity);
            }
        }

        // Tab Select & Close Message
        for (user_key, message) in events.read::<TabActionChannel, TabCloseMessage>() {
            let tab_id = message.tab_id;
            tab_manager.queue_close_tab(user_key, tab_id);
        }
    }
}
