use bevy_ecs::system::Resource;
use bevy_ecs::world::{Mut, World};
use naia_bevy_server::UserKey;
use vortex_proto::messages::{ChangelistAction, ChangelistMessage};
use crate::resources::user_manager::UserInfo;
use crate::resources::{GitManager, UserManager};

#[derive(Resource)]
pub struct ChangelistManager {
    messages: Vec<(UserKey, ChangelistMessage)>,
}

impl Default for ChangelistManager {
    fn default() -> Self {
        Self {
            messages: Vec::new(),
        }
    }
}

impl ChangelistManager {
    pub fn queue_changelist_message(&mut self, user_key: UserKey, message: ChangelistMessage) {
        self.messages.push((user_key, message));
    }

    pub fn process_messages(&mut self, world: &mut World, git_manager: &mut GitManager) {
        for (user_key, message) in self.messages.drain(..) {
            self.process_message(world, git_manager, user_key, message);
        }
    }

    fn process_message(
        &mut self,
        world: &mut World,
        git_manager: &mut GitManager,
        user_key: UserKey,
        message: ChangelistMessage
    ) {
        match message.action {
            ChangelistAction::Commit => {
                git_manager.commit_changelist_entry(
                    world,
                    user_key,
                    message,
                );
            }
            ChangelistAction::Rollback => {
                git_manager.rollback_changelist_entry(
                    world,
                    user_key,
                    message,
                );
            }
        }
    }
}

pub fn changelist_manager_process(world: &mut World) {
    world.resource_scope(|world, mut changelist_manager: Mut<ChangelistManager>| {
        if changelist_manager.messages.is_empty() {
            return;
        }
        world.resource_scope(|world, mut git_manager: Mut<GitManager>| {
            changelist_manager.process_messages(world, &mut git_manager);
        });
    });
}