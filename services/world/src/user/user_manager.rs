use std::collections::HashMap;

use bevy_ecs::system::Resource;

use auth_server_types::UserId;
use naia_bevy_server::UserKey;
use social_server_types::LobbyId;

use crate::user::{user_data::UserData, user_login_token_store::UserLoginTokenStore};

#[derive(Resource)]
pub struct UserManager {
    login_token_store: UserLoginTokenStore,
    users: HashMap<UserKey, UserData>,
}

impl Default for UserManager {
    fn default() -> Self {
        Self {
            login_token_store: UserLoginTokenStore::new(),
            users: HashMap::new(),
        }
    }
}

impl UserManager {

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

    pub fn add_user(&mut self, user_key: &UserKey, user_data: UserData) {
        self.users.insert(user_key.clone(), user_data);
    }

    pub fn recv_login_token(
        &mut self,
        lobby_id: &LobbyId,
        login_tokens: &Vec<(String, u16, Vec<(UserId, String)>)>,
    ) {
        self.login_token_store.recv_login_token(lobby_id, login_tokens);
    }

    pub fn spend_login_token(&mut self, token: &str) -> Option<UserData> {
        self.login_token_store.spend_login_token(token)
    }

    pub fn reset(&mut self) {

        // clear login tokens
        self.login_token_store.clear();

        // clear users
        self.users.clear();
    }
}
