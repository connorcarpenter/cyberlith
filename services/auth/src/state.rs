use std::collections::HashMap;

use log::{info, warn};

use auth_server_db::{AuthServerDbError, DatabaseManager, User, UserId, UserRole};
use auth_server_http_proto::{UserRegisterConfirmRequest, UserRegisterRequest};
use crypto::U32Token;

use crate::{error::AuthServerError, emails::EmailCatalog};

pub struct State {
    database_manager: DatabaseManager,
    email_catalog: EmailCatalog,
    temp_regs: HashMap<RegisterToken, TempRegistration>,
    username_to_id_map: HashMap<String, Option<UserId>>,
    email_to_id_map: HashMap<String, Option<UserId>>,
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
            temp_regs: HashMap::new(),
            username_to_id_map,
            email_to_id_map,
        }
    }

    pub fn user_register(&mut self, request: UserRegisterRequest) -> Result<(), AuthServerError> {

        // TODO: validate data?
        // TODO: hash password?
        // TODO: expire registration token?

        if self.username_to_id_map.contains_key(&request.username) {
            return Err(AuthServerError::UsernameAlreadyExists);
        }
        if self.email_to_id_map.contains_key(&request.email) {
            return Err(AuthServerError::EmailAlreadyExists);
        }

        let mut reg_token = RegisterToken::gen_random();
        while self.temp_regs.contains_key(&reg_token) {
            reg_token = RegisterToken::gen_random();
        }

        let temp_reg = TempRegistration::from(request);

        let email_subject = "Cyberlith Email Verification";
        let sending_email = "cyberlithgame@gmail.com";
        let username = temp_reg.name.clone();
        let user_email: String = temp_reg.email.clone();
        let reg_token_str = reg_token.value.as_string();
        let link_url = format!("register_token={}", reg_token_str); // TODO: replace with working URL

        info!("sending registration token to user's email: {:?}", &user_email);

        let text_msg = self.email_catalog.register_verification_txt(&username, &link_url);
        let html_msg = self.email_catalog.register_verification_html(&username, &link_url);

        match email::send(
            sending_email,
            &user_email,
            email_subject,
            &text_msg,
            &html_msg,
        ) {
            Ok(_response) => {
                info!("email send success!");

                self.temp_regs.insert(reg_token, temp_reg);
                self.username_to_id_map.insert(username, None);
                self.email_to_id_map.insert(user_email, None);

                return Ok(());
            }
            Err(err) => {
                warn!("email send failed: {:?}", err);
                return Err(AuthServerError::EmailSendFailed(err.to_string()));
            }
        }
    }

    pub fn user_register_confirm(&mut self, request: UserRegisterConfirmRequest) -> Result<(), AuthServerError> {

        let Some(reg_token) = U32Token::from_str(&request.register_token) else {
            return Err(AuthServerError::RegisterTokenSerdeError);
        };
        let reg_token = RegisterToken::from(reg_token);
        let Some(temp_reg) = self.temp_regs.remove(&reg_token) else {
            return Err(AuthServerError::RegisterTokenNotFound);
        };

        let new_user = User::new(&temp_reg.name, &temp_reg.email, &temp_reg.password, UserRole::Free);
        let new_user_id = self.database_manager.create_user(new_user).map_err(|err| {
            match err {
                AuthServerDbError::InsertedDuplicateUserId => AuthServerError::InsertedDuplicateUserId,
            }
        })?;

        // add to username -> id map
        let Some(id_opt) = self.username_to_id_map.get_mut(&temp_reg.name) else {
            return Err(AuthServerError::Unknown("username not found AFTER register confirm".to_string()));
        };
        if id_opt.is_some() {
            return Err(AuthServerError::Unknown("username already exists AFTER register confirm".to_string()));
        }
        *id_opt = Some(new_user_id);

        // add to email -> id map
        let Some(id_opt) = self.email_to_id_map.get_mut(&temp_reg.email) else {
            return Err(AuthServerError::Unknown("email not found AFTER register confirm".to_string()));
        };
        if id_opt.is_some() {
            return Err(AuthServerError::Unknown("email already exists AFTER register confirm".to_string()));
        }
        *id_opt = Some(new_user_id);

        let new_user_id: u64 = new_user_id.into();
        info!("new user created: {:?} - {:?}", new_user_id, temp_reg.name);

        Ok(())
    }
}

#[derive(Eq, PartialEq, Hash)]
struct RegisterToken {
    value: U32Token,
}

impl From<U32Token> for RegisterToken {
    fn from(value: U32Token) -> Self {
        Self { value }
    }
}

impl RegisterToken {
    pub fn gen_random() -> Self {
        Self {
            value: U32Token::gen_random(),
        }
    }
}

struct TempRegistration {
    pub name: String,
    pub email: String,
    pub password: String,
}

impl From<UserRegisterRequest> for TempRegistration {
    fn from(req: UserRegisterRequest) -> Self {
        Self {
            name: req.username,
            email: req.email,
            password: req.password,
        }
    }
}