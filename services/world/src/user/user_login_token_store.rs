use std::collections::HashMap;

use auth_server_types::UserId;
use social_server_types::LobbyId;

use crate::user::UserData;

pub(crate) struct UserLoginTokenStore {
    login_tokens: HashMap<String, UserData>,
}

impl UserLoginTokenStore {
    pub fn new() -> Self {
        Self {
            login_tokens: HashMap::new(),
        }
    }

    pub fn clear(&mut self) {
        self.login_tokens.clear();
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
}