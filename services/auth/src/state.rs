use std::collections::HashMap;

use auth_server_db::{DatabaseManager, UserId};

use crate::{types::{UserData, RefreshToken, RegisterToken, AccessToken, TempRegistration}, emails::EmailCatalog};

pub struct State {
    pub(crate) email_catalog: EmailCatalog,

    pub(crate) database_manager: DatabaseManager,
    pub(crate) username_to_id_map: HashMap<String, Option<UserId>>,
    pub(crate) email_to_id_map: HashMap<String, Option<UserId>>,

    register_tokens: HashMap<RegisterToken, TempRegistration>,
    access_tokens: HashMap<AccessToken, UserId>,
    refresh_tokens: HashMap<RefreshToken, UserId>,
    user_data: HashMap<UserId, UserData>,
}

impl State {
    pub fn new() -> Self {

        let database_manager = DatabaseManager::init();
        let mut username_to_id_map = HashMap::new();
        let mut email_to_id_map = HashMap::new();
        let mut user_data_map = HashMap::new();

        for (id, user) in database_manager.list_users() {
            username_to_id_map.insert(user.username().to_string(), Some(*id));
            email_to_id_map.insert(user.email().to_string(), Some(*id));
            user_data_map.insert(*id, UserData::new());
        }

        Self {
            email_catalog: EmailCatalog::new(),

            database_manager,
            username_to_id_map,
            email_to_id_map,
            user_data: user_data_map,

            register_tokens: HashMap::new(),
            access_tokens: HashMap::new(),
            refresh_tokens: HashMap::new(),
        }
    }

    // register tokens
    pub(crate) fn create_new_register_token(&self) -> RegisterToken {
        let mut token = RegisterToken::gen_random();
        while self.register_tokens.contains_key(&token) {
            token = RegisterToken::gen_random();
        }
        token
    }

    pub(crate) fn store_register_token(&mut self, token: RegisterToken, temp_reg: TempRegistration) {
        self.register_tokens.insert(token, temp_reg);
    }

    pub(crate) fn remove_register_token(&mut self, token: &RegisterToken) -> Option<TempRegistration> {
        self.register_tokens.remove(token)
    }

    // refresh and access tokens
    pub(crate) fn user_new_login_gen_tokens(&mut self, user_id: &UserId) -> (RefreshToken, AccessToken) {
        let access_token = self.create_and_store_access_token(user_id);
        let refresh_token = self.create_and_store_refresh_token(user_id);
        (refresh_token, access_token)
    }

    fn create_and_store_refresh_token(&mut self, user_id: &UserId) -> RefreshToken {
        let mut token = RefreshToken::gen_random();
        while self.refresh_tokens.contains_key(&token) {
            token = RefreshToken::gen_random();
        }

        self.refresh_tokens.insert(token, *user_id);

        // insert into userdata
        self.user_data.get_mut(user_id).unwrap().current_refresh_token = Some(token);

        token
    }

    fn create_and_store_access_token(&mut self, user_id: &UserId) -> AccessToken {
        let mut token = AccessToken::gen_random();
        while self.access_tokens.contains_key(&token) {
            token = AccessToken::gen_random();
        }

        self.access_tokens.insert(token, *user_id);

        // insert into userdata
        self.user_data.get_mut(user_id).unwrap().current_access_token = Some(token);

        token
    }

    pub(crate) fn has_access_token(&self, access_token: &AccessToken) -> bool {
        self.access_tokens.contains_key(access_token)
    }
}

