use std::collections::HashSet;

use auth_server_types::UserId;

use crate::session_servers::SessionServerId;

pub(crate) enum UserPatch {
    Add(UserId),
    Remove(SessionServerId, UserId),
}

pub struct UsersState {

    users: HashSet<UserId>,

    // the session server id here is the SENDER not the RECEIVER
    outgoing_patches: Vec<UserPatch>,
}

impl UsersState {
    pub fn new() -> Self {
        Self {
            users: HashSet::new(),
            outgoing_patches: Vec::new(),
        }
    }

    pub fn is_user_online(&self, user_id: &UserId) -> bool {
        self.users.contains(user_id)
    }

    pub fn connect_user(&mut self, user_id: &UserId) {
        self.users.insert(*user_id);

        self.outgoing_patches.push(UserPatch::Add(*user_id));
    }

    pub fn disconnect_user(&mut self, sending_session_server_id: SessionServerId, user_id: UserId) {
        self.users.remove(&user_id);

        self.outgoing_patches.push(UserPatch::Remove(sending_session_server_id, user_id));
    }

    pub fn get_online_users(&self) -> Vec<UserId> {
        self.users.iter().map(|id| *id).collect()
    }

    pub fn take_patches(
        &mut self,
    ) -> Vec<UserPatch> {
        std::mem::take(&mut self.outgoing_patches)
    }
}
