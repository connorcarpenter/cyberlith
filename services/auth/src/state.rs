use std::collections::HashMap;

use log::{info, warn};

use auth_server_db::{DatabaseManager, User, UserRole};
use auth_server_http_proto::{UserRegisterConfirmRequest, UserRegisterRequest};
use crypto::U32Token;

use crate::emails::EmailCatalog;
use crate::error::AuthServerError;

pub struct State {
    database_manager: DatabaseManager,
    email_catalog: EmailCatalog,

    temp_regs: HashMap<RegisterToken, TempRegistration>,
}

impl State {
    pub fn new() -> Self {
        Self {
            email_catalog: EmailCatalog::new(),
            database_manager: DatabaseManager::init(),
            temp_regs: HashMap::new(),
        }
    }

    pub fn user_register(&mut self, request: UserRegisterRequest) -> Result<(), AuthServerError> {

        // TODO: validate data?
        // TODO: hash password?
        // TODO: check if user already exists?
        // TODO: expire registration token?

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
        let link_url = format!("http://localhost:4000/?register_token={}", reg_token_str);

        self.temp_regs.insert(reg_token, temp_reg);

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
            return Err(AuthServerError::TokenSerdeError);
        };
        let reg_token = RegisterToken::from(reg_token);
        let Some(temp_reg) = self.temp_regs.remove(&reg_token) else {
            return Err(AuthServerError::TokenNotFound);
        };

        let new_user = User::new(&temp_reg.name, &temp_reg.email, &temp_reg.password, UserRole::Free);
        let new_user_id = self.database_manager.create_user(new_user);
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