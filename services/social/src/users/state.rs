use std::collections::HashMap;

use auth_server_types::UserId;

use crate::session_servers::SessionServerId;

pub(crate) enum UserPatch {
    Add(UserId, String),
    Remove(SessionServerId, UserId),
}

pub struct UsersState {
    // id, name
    users: HashMap<UserId, String>,

    // the session server id here is the SENDER not the RECEIVER
    outgoing_patches: Vec<UserPatch>,
}

impl UsersState {
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
            outgoing_patches: Vec::new(),
        }
    }

    pub fn connect_user(&mut self, user_id: &UserId, user_name: &str) {
        self.users.insert(*user_id, user_name.to_string());

        self.outgoing_patches.push(UserPatch::Add(*user_id, user_name.to_string()));
    }

    pub fn disconnect_user(&mut self, sending_session_server_id: SessionServerId, user_id: UserId) {
        self.users.remove(&user_id);

        self.outgoing_patches.push(UserPatch::Remove(sending_session_server_id, user_id));
    }

    pub fn get_present_users(&self) -> Vec<(UserId, String)> {
        self.users.iter().map(|(id, name)| (*id, name.clone())).collect()
    }

    pub fn take_patches(
        &mut self,
    ) -> Vec<UserPatch> {
        std::mem::take(&mut self.outgoing_patches)
    }
}
