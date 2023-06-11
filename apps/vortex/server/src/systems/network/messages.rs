use bevy_ecs::{event::EventReader, system::{Query, Res, ResMut, Commands}};

use naia_bevy_server::{events::MessageEvents, Server};

use vortex_proto::{channels::ChangelistActionChannel, messages::{ChangelistMessage, ChangelistAction}, components::ChangelistEntry};

use crate::resources::{GitManager, UserManager};

pub fn message_events(
    mut commands: Commands,
    mut server: Server,
    mut event_reader: EventReader<MessageEvents>,
    user_manager: Res<UserManager>,
    mut git_manager: ResMut<GitManager>,
    query: Query<&ChangelistEntry>,
) {
    for events in event_reader.iter() {
        for (user_key, message) in events.read::<ChangelistActionChannel, ChangelistMessage>() {

            let Some(user) = user_manager.user_info(&user_key) else {
                panic!("user not found!");
            };

            match message.action {
                ChangelistAction::CommitAll => {
                    git_manager.commit_entire_changelist(&mut commands, &mut server, user, &query);
                }
                ChangelistAction::CommitSingle => {
                    let Some(entity) = message.entity.get(&server) else {
                        panic!("no entity!")
                    };
                    git_manager.commit_changelist_entry(&mut commands, &mut server, user, &entity, &query);
                }
                ChangelistAction::Rollback => {
                    let Some(entity) = message.entity.get(&server) else {
                        panic!("no entity!")
                    };
                    git_manager.rollback_changelist_entry(&mut commands, &mut server, &user_key, user, &entity, &query);
                }
            }
        }
    }
}