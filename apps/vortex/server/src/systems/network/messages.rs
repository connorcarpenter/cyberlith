use bevy_ecs::{
    event::EventReader,
    system::{Commands, Query, ResMut},
};
use bevy_log::info;

use naia_bevy_server::{events::MessageEvents, Server};

use vortex_proto::{
    channels::{FileActionChannel, TabActionChannel},
    messages::{ChangelistMessage, TabActionMessage, TabActionMessageType, TabOpenMessage},
    resources::FileEntryKey,
};
use vortex_proto::messages::FileBindMessage;

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
    key_q: Query<&FileEntryKey>,
) {
    for events in event_reader.iter() {

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
                tab_manager.open_tab(
                    &mut commands,
                    &mut server,
                    &mut user_manager,
                    &mut git_manager,
                    &mut shape_manager,
                    &key_q,
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

        // File Bind Message
        for (user_key, message) in events.read::<FileActionChannel, FileBindMessage>() {
            let file_entity = message.file_entity.get(&server).unwrap();
            let dependency_entity = message.dependency_entity.get(&server).unwrap();

            let project_key = user_manager.user_session_data(&user_key).unwrap().project_key().unwrap();
            let file_key = key_q.get(file_entity).unwrap().clone();
            let dependency_key = key_q.get(dependency_entity).unwrap().clone();

            let project = git_manager.project_mut(&project_key).unwrap();
            project.file_add_dependency(&file_key, &dependency_key);

            git_manager.on_client_modify_file(&mut commands, &mut server, &project_key, &file_key);

            info!("received FileBindMessage(file: `{:?}`, dependency: `{:?}`)", file_key.name(), dependency_key.name());
        }
    }
}
