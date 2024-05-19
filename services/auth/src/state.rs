use std::collections::HashMap;

use auth_server_db::DatabaseManager;
use auth_server_http_proto::{AccessToken, RefreshToken, RegisterToken, ResetPasswordToken};
use auth_server_types::UserId;
use logging::info;

use crate::{
    emails::EmailCatalog,
    expire_manager::{ExpireEvent, ExpireManager},
    types::{TempRegistration, UserData},
};

pub struct State {
    pub(crate) email_catalog: EmailCatalog,

    pub(crate) database_manager: DatabaseManager,
    pub(crate) username_to_id_map: HashMap<String, Option<UserId>>,
    pub(crate) email_to_id_map: HashMap<String, Option<UserId>>,

    register_tokens: HashMap<RegisterToken, TempRegistration>,
    reset_password_tokens: HashMap<ResetPasswordToken, UserId>,
    access_tokens: HashMap<AccessToken, UserId>,
    refresh_tokens: HashMap<RefreshToken, UserId>,
    user_data: HashMap<UserId, UserData>,
    expire_manager: ExpireManager,
}

impl State {
    pub fn new() -> Self {
        let database_manager = DatabaseManager::init();
        let mut username_to_id_map = HashMap::new();
        let mut email_to_id_map = HashMap::new();
        let mut user_data_map = HashMap::new();

        for (id, user) in database_manager.list_users() {
            username_to_id_map.insert(user.username().to_string(), Some(id));
            email_to_id_map.insert(user.email().to_string(), Some(id));
            user_data_map.insert(id, UserData::new());
        }

        Self {
            email_catalog: EmailCatalog::new(),

            database_manager,
            username_to_id_map,
            email_to_id_map,
            user_data: user_data_map,

            register_tokens: HashMap::new(),
            reset_password_tokens: HashMap::new(),
            access_tokens: HashMap::new(),
            refresh_tokens: HashMap::new(),
            expire_manager: ExpireManager::new(),
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

    pub(crate) fn store_register_token(
        &mut self,
        token: RegisterToken,
        temp_reg: TempRegistration,
    ) {
        self.register_tokens.insert(token, temp_reg);
        self.expire_manager.insert_register_token(&token);
    }

    pub(crate) fn remove_register_token(
        &mut self,
        token: &RegisterToken,
    ) -> Option<TempRegistration> {
        self.register_tokens.remove(token)
    }

    // refresh and access tokens
    pub(crate) fn user_new_login_gen_tokens(
        &mut self,
        user_id: &UserId,
    ) -> (RefreshToken, AccessToken) {
        let access_token = self.create_and_store_access_token(user_id);
        let refresh_token = self.create_and_store_refresh_token(user_id);
        (refresh_token, access_token)
    }

    pub(crate) fn init_user_data(&mut self, user_id: &UserId) {
        self.user_data.insert(*user_id, UserData::new());
    }

    fn create_and_store_refresh_token(&mut self, user_id: &UserId) -> RefreshToken {
        let mut token = RefreshToken::gen_random();
        while self.refresh_tokens.contains_key(&token) {
            token = RefreshToken::gen_random();
        }

        self.refresh_tokens.insert(token, *user_id);
        self.expire_manager.insert_refresh_token(&token);

        // insert into userdata
        self.user_data
            .get_mut(user_id)
            .unwrap()
            .current_refresh_token = Some(token);

        token
    }

    pub(crate) fn create_and_store_access_token(&mut self, user_id: &UserId) -> AccessToken {
        let mut token = AccessToken::gen_random();
        while self.access_tokens.contains_key(&token) {
            token = AccessToken::gen_random();
        }

        self.access_tokens.insert(token, *user_id);
        self.expire_manager.insert_access_token(&token);

        // insert into userdata
        self.user_data
            .get_mut(user_id)
            .unwrap()
            .current_access_token = Some(token);

        token
    }

    pub(crate) fn has_refresh_token(&self, refresh_token: &RefreshToken) -> bool {
        self.refresh_tokens.contains_key(refresh_token)
    }

    pub(crate) fn get_access_token(&self, access_token: &AccessToken) -> Option<&UserId> {
        self.access_tokens.get(access_token)
    }

    pub(crate) fn get_user_id_by_refresh_token(
        &self,
        refresh_token: &RefreshToken,
    ) -> Option<&UserId> {
        self.refresh_tokens.get(refresh_token)
    }

    // for username recovery
    pub(crate) fn get_user_name_by_email(&self, email: &str) -> String {
        let user_id = self.email_to_id_map.get(email).unwrap().unwrap();
        self.database_manager
            .get_user(&user_id)
            .unwrap()
            .username()
            .to_string()
    }

    pub(crate) fn get_user_id_by_email(&self, email: &str) -> UserId {
        self.email_to_id_map.get(email).unwrap().unwrap()
    }

    // reset password tokens
    pub(crate) fn create_new_reset_password_token(&self) -> ResetPasswordToken {
        let mut token = ResetPasswordToken::gen_random();
        while self.reset_password_tokens.contains_key(&token) {
            token = ResetPasswordToken::gen_random();
        }
        token
    }

    pub(crate) fn store_reset_password_token(
        &mut self,
        user_id: UserId,
        token: ResetPasswordToken,
    ) {
        self.reset_password_tokens.insert(token, user_id);
        self.expire_manager.insert_reset_password_token(&token);
    }

    pub(crate) fn remove_reset_password_token(
        &mut self,
        token: &ResetPasswordToken,
    ) -> Option<UserId> {
        self.reset_password_tokens.remove(token)
    }

    pub(crate) fn set_user_password(&mut self, user_id: UserId, new_password: String) {
        self.database_manager.get_user_mut(&user_id, |user| {
            user.set_password(&new_password);
        });
    }

    pub(crate) fn clear_expired_tokens(&mut self) {
        for event in self.expire_manager.clear_expired() {
            match event {
                ExpireEvent::AccessToken(token) => {
                    info!("Access token expired: {:?}", token);
                    let Some(user_id) = self.access_tokens.remove(&token) else {
                        continue;
                    };
                    let Some(user) = self.user_data.get_mut(&user_id) else {
                        continue;
                    };
                    user.current_access_token = None;
                }
                ExpireEvent::RefreshToken(token) => {
                    info!("Refresh token expired: {:?}", token);
                    let Some(user_id) = self.refresh_tokens.remove(&token) else {
                        continue;
                    };
                    let Some(user) = self.user_data.get_mut(&user_id) else {
                        continue;
                    };
                    user.current_refresh_token = None;
                }
                ExpireEvent::RegisterToken(token) => {
                    info!("Register token expired: {:?}", token);
                    self.register_tokens.remove(&token);
                }
                ExpireEvent::ResetPasswordToken(token) => {
                    info!("Reset password token expired: {:?}", token);
                    self.reset_password_tokens.remove(&token);
                }
            }
        }
    }
}
