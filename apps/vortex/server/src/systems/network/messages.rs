use bevy_ecs::{
    event::EventReader,
    system::{Commands, Query, ResMut},
};
use bevy_log::info;

use naia_bevy_server::{events::MessageEvents, Server};

use vortex_proto::{
    channels::{ChangelistActionChannel, TabActionChannel},
    messages::{ChangelistMessage, TabActionMessage, TabActionMessageType, TabOpenMessage},
    resources::FileEntryKey,
};

use crate::resources::{ChangelistManager, GitManager, ShapeManager, TabManager, UserManager};

pub fn message_events(
    mut commands: Commands,
    mut server: Server,
    mut event_reader: EventReader<MessageEvents>,
    mut user_manager: ResMut<UserManager>,
    mut git_manager: ResMut<GitManager>,
    mut tab_manager: ResMut<TabManager>,
    mut cl_manager: ResMut<ChangelistManager>,
    mut shape_manager: ResMut<ShapeManager>,
    key_query: Query<&FileEntryKey>,
) {
    for events in event_reader.iter() {
        for (user_key, message) in events.read::<ChangelistActionChannel, ChangelistMessage>() {
            info!("received ChangelistMessage with action: {:?}", message.action);

            cl_manager.queue_changelist_message(user_key, message);
        }

        // Tab Open Message
        for (user_key, message) in events.read::<TabActionChannel, TabOpenMessage>() {
            let tab_id = message.tab_id;
            if let Some(file_entity) = message.file_entity.get(&server) {
                tab_manager.open_tab(
                    &mut commands,
                    &mut server,
                    &mut user_manager,
                    &mut git_manager,
                    &mut shape_manager,
                    &key_query,
                    &user_key,
                    &tab_id,
                    &file_entity,
                );
            }
        }

        // Tab Select & Close Message
        for (user_key, message) in events.read::<TabActionChannel, TabActionMessage>() {
            let tab_id = message.tab_id;
            match message.action {
                TabActionMessageType::Select => {
                    user_manager.select_tab(&user_key, &tab_id);
                }
                TabActionMessageType::Close => {
                    tab_manager.queue_close_tab(user_key, tab_id);
                }
            }
        }
    }
}
