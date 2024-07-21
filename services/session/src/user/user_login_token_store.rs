use std::collections::HashMap;

use auth_server_types::UserId;

pub(crate) struct UserLoginTokenStore {
    login_tokens: HashMap<String, UserId>,
}

impl UserLoginTokenStore {
    pub fn new() -> Self {
        Self {
            login_tokens: HashMap::new(),
        }
    }

    pub fn recv_login_token(&mut self, user_id: &UserId, token: &str) {
        self.login_tokens.insert(token.to_string(), user_id.clone());
    }

    pub fn spend_login_token(&mut self, token: &str) -> Option<UserId> {
        self.login_tokens.remove(token)
    }
}