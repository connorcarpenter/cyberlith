use std::collections::HashMap;

use auth_server_types::UserId;
use social_server_types::LobbyId;

use crate::session_servers::SessionServerId;

pub(crate) enum UserPatch {
    Add(UserId),
    Remove(SessionServerId, UserId),
}

struct UserData {
    session_server_id: SessionServerId,
    lobby_id: Option<LobbyId>,
}

impl UserData {
    pub fn new(session_server_id: &SessionServerId) -> Self {
        Self {
            session_server_id: *session_server_id,
            lobby_id: None,
        }
    }
}

pub struct UsersState {
    users: HashMap<UserId, UserData>,

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

    pub fn is_user_online(&self, user_id: &UserId) -> bool {
        self.users.contains_key(user_id)
    }

    pub fn connect_user(&mut self, user_id: &UserId, session_server_id: &SessionServerId) {
        self.users.insert(*user_id, UserData::new(session_server_id));

        self.outgoing_patches.push(UserPatch::Add(*user_id));
    }

    pub fn disconnect_user(&mut self, sending_session_server_id: SessionServerId, user_id: UserId) {
        self.users.remove(&user_id);

        self.outgoing_patches
            .push(UserPatch::Remove(sending_session_server_id, user_id));
    }

    pub fn get_online_users(&self) -> Vec<UserId> {
        self.users.iter().map(|(id, _)| *id).collect()
    }

    pub fn take_patches(&mut self) -> Vec<UserPatch> {
        std::mem::take(&mut self.outgoing_patches)
    }

    pub fn get_user_lobby_id(&self, user_id: &UserId) -> Option<LobbyId> {
        let user_data = self.users.get(user_id)?;
        user_data.lobby_id
    }

    pub fn get_user_session_server_id(&self, user_id: &UserId) -> SessionServerId {
        self.users.get(user_id).unwrap().session_server_id
    }

    pub fn user_joins_lobby(&mut self, user_id: &UserId, lobby_id: &LobbyId) {
        let user_data = self.users.get_mut(user_id).unwrap();
        user_data.lobby_id = Some(*lobby_id);
    }

    pub fn user_leaves_lobby(&mut self, user_id: &UserId) -> LobbyId {
        let user_data = self.users.get_mut(user_id).unwrap();
        let output = user_data.lobby_id.unwrap();
        user_data.lobby_id = None;
        output
    }
}
