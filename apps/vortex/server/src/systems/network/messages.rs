use bevy_ecs::{event::EventReader, system::{Commands, Query, Res, ResMut}};
use bevy_log::info;
use naia_bevy_server::{events::MessageEvents, Server};

use vortex_proto::{channels::{ChangelistActionChannel, TabActionChannel}, components::ChangelistEntry, messages::{ChangelistAction, ChangelistMessage, TabActionMessage, TabActionMessageType, TabOpenMessage}};

use crate::resources::{GitManager, TabManager, UserManager};

pub fn message_events(
    mut commands: Commands,
    mut server: Server,
    mut event_reader: EventReader<MessageEvents>,
    user_manager: Res<UserManager>,
    mut git_manager: ResMut<GitManager>,
    mut tab_manager: ResMut<TabManager>,
    cl_query: Query<&ChangelistEntry>,
) {
    for events in event_reader.iter() {
        for (user_key, message) in events.read::<ChangelistActionChannel, ChangelistMessage>() {

            info!("received ChangelistMessage");

            let Some(user) = user_manager.user_info(&user_key) else {
                panic!("user not found!");
            };

            match message.action {
                ChangelistAction::Commit => {
                    let Some(entity) = message.entity.get(&server) else {
                        panic!("no entity!")
                    };
                    let Some(commit_message) = message.commit_message else {
                        panic!("no commit message!")
                    };
                    git_manager.commit_changelist_entry(&mut commands, &mut server, user, &commit_message, &entity, &cl_query);
                }
                ChangelistAction::Rollback => {
                    let Some(entity) = message.entity.get(&server) else {
                        panic!("no entity!")
                    };
                    git_manager.rollback_changelist_entry(&mut commands, &mut server, &user_key, user, &entity, &cl_query);
                }
            }
        }

        // Tab Open Message
        for (user_key, message) in events.read::<TabActionChannel, TabOpenMessage>() {
            let tab_id = message.tab_id;
            if let Some(file_entity) = message.file_entity.get(&server) {
                tab_manager.open_tab(&mut commands, &mut server, &mut git_manager, &user_key, &tab_id, &file_entity);
            }
        }

        // Tab Select & Close Message
        for (user_key, message) in events.read::<TabActionChannel, TabActionMessage>() {
            let tab_id = message.tab_id;
            match message.action {
                TabActionMessageType::Select => {
                    tab_manager.select_tab(&mut commands, &mut server, &user_key, &tab_id);
                }
                TabActionMessageType::Close => {
                    tab_manager.close_tab(&mut commands, &mut server, &mut git_manager, &user_key, &tab_id);
                }
            }
        }
    }
}