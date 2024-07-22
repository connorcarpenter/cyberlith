use std::collections::HashMap;

use bevy_ecs::system::Resource;

use auth_server_types::UserId;
use naia_bevy_server::UserKey;
use social_server_types::LobbyId;

pub struct UserData {
    pub user_id: UserId,
    pub lobby_id: LobbyId,
    pub session_server_addr: String,
    pub session_server_port: u16,
}

impl UserData {
    fn new(user_id: UserId, lobby_id: LobbyId, session_server_addr: &str, session_server_port: u16) -> Self {
        Self {
            user_id,
            lobby_id,
            session_server_addr: session_server_addr.to_string(),
            session_server_port,
        }
    }
}

#[derive(Resource)]
pub struct UserManager {
    login_tokens: HashMap<String, UserData>,
    users: HashMap<UserKey, UserData>,
}

impl Default for UserManager {
    fn default() -> Self {
        Self {
            login_tokens: HashMap::new(),
            users: HashMap::new(),
        }
    }

}

impl UserManager {

    pub fn disconnect_region_server(&mut self) {

        // clear login tokens
        self.login_tokens.clear();

        // clear users
        self.users.clear();
    }

    pub fn recv_login_token(
        &mut self,
        lobby_id: &LobbyId,
        login_tokens: &Vec<(String, u16, Vec<(UserId, String)>)>,
    ) {
        for (session_server_addr, session_server_port, tokens) in login_tokens {
            for (user_id, token) in tokens {
                self.login_tokens.insert(
                    token.to_string(),
                    UserData::new(*user_id, *lobby_id, session_server_addr, *session_server_port),
                );
            }
        }
    }

    pub fn spend_login_token(&mut self, token: &str) -> Option<UserData> {
        self.login_tokens.remove(token)
    }

    pub fn add_user(&mut self, user_key: &UserKey, user_data: UserData) {
        self.users.insert(user_key.clone(), user_data);
    }

    pub fn get_user_id(&self, user_key: &UserKey) -> Option<UserId> {
        let user_data = self.users.get(user_key)?;
        Some(user_data.user_id)
    }

    pub fn get_user_lobby_id(&self, user_key: &UserKey) -> Option<LobbyId> {
        let user_data = self.users.get(user_key)?;
        Some(user_data.lobby_id)
    }

    pub fn get_user_session_server(&self, user_key: &UserKey) -> Option<(String, u16)> {
        let user_data = self.users.get(user_key)?;
        Some((
            user_data.session_server_addr.clone(),
            user_data.session_server_port,
        ))
    }
}
