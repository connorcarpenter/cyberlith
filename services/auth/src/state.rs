use std::collections::HashMap;

use auth_server_db::{DatabaseManager, UserId};

use crate::{types::{RegisterToken, AccessToken, TempRegistration}, emails::EmailCatalog};

pub struct State {
    pub(crate) database_manager: DatabaseManager,
    pub(crate) email_catalog: EmailCatalog,
    registration_tokens: HashMap<RegisterToken, TempRegistration>,
    pub(crate) access_tokens: HashMap<AccessToken, UserId>,
    pub(crate) username_to_id_map: HashMap<String, Option<UserId>>,
    pub(crate) email_to_id_map: HashMap<String, Option<UserId>>,
}

impl State {
    pub fn new() -> Self {

        let database_manager = DatabaseManager::init();
        let mut username_to_id_map = HashMap::new();
        let mut email_to_id_map = HashMap::new();

        for (id, user) in database_manager.list_users() {
            username_to_id_map.insert(user.username().to_string(), Some(*id));
            email_to_id_map.insert(user.email().to_string(), Some(*id));
        }

        Self {
            email_catalog: EmailCatalog::new(),
            database_manager,
            registration_tokens: HashMap::new(),
            access_tokens: HashMap::new(),
            username_to_id_map,
            email_to_id_map,
        }
    }

    pub(crate) fn create_new_register_token(&self) -> RegisterToken {
        let mut token = RegisterToken::gen_random();
        while self.registration_tokens.contains_key(&token) {
            token = RegisterToken::gen_random();
        }
        token
    }

    pub(crate) fn store_register_token(&mut self, token: RegisterToken, temp_reg: TempRegistration) {
        self.registration_tokens.insert(token, temp_reg);
    }

    pub(crate) fn remove_register_token(&mut self, token: &RegisterToken) -> Option<TempRegistration> {
        self.registration_tokens.remove(token)
    }

    pub(crate) fn create_and_store_new_access_token(&mut self, user_id: &UserId) -> AccessToken {
        let mut token = AccessToken::gen_random();
        while self.access_tokens.contains_key(&token) {
            token = AccessToken::gen_random();
        }

        self.access_tokens.insert(token, *user_id);

        token
    }
}

