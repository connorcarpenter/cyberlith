use std::collections::HashMap;
use log::info;

use auth_server_db::{DatabaseManager, User, UserRole};
use auth_server_http_proto::{UserRegisterConfirmRequest, UserRegisterRequest};
use crypto::U32Token;

pub struct State {
    database_manager: DatabaseManager,

    temp_regs: HashMap<RegisterToken, TempRegistration>,
}

impl State {
    pub fn new() -> Self {
        Self {
            database_manager: DatabaseManager::init(),
            temp_regs: HashMap::new(),
        }
    }

    pub fn user_register(&mut self, request: UserRegisterRequest) {

        // TODO: validate data?
        // TODO: hash password?
        // TODO: check if user already exists?
        // TODO: send email confirmation?
        // TODO: expire registration token?
        // TODO: return result?

        info!("storing temporary registration");
        let reg_token = RegisterToken::gen_random();
        let temp_reg = TempRegistration::from(request);
        let email: String = temp_reg.email.clone();

        if self.temp_regs.contains_key(&reg_token) {
            panic!("register token collision");
        }
        self.temp_regs.insert(reg_token, temp_reg);

        info!("sending token to user's email: {:?}", &email);
        let _ = email::send(
            "cyberlithgame@gmail.com",
            "connorcarpenter@gmail.com",
            "Cyber Test Email",
            "Hello, this is a test",
            "<h1>Hello</h1><p>This is a test email with HTML content.</p>"
        );
        info!("success!");
    }

    pub fn user_register_confirm(&mut self, request: UserRegisterConfirmRequest) {

        let Some(reg_token) = U32Token::from_str(&request.register_token) else {
            panic!("invalid register token");
        };
        let reg_token = RegisterToken::from(reg_token);
        let Some(temp_reg) = self.temp_regs.remove(&reg_token) else {
            panic!("register token not found");
        };

        let new_user = User::new(&temp_reg.name, &temp_reg.email, &temp_reg.password, UserRole::Free);
        let new_user_id = self.database_manager.create_user(new_user);
        let new_user_id: u64 = new_user_id.into();
        info!("new user created: {:?} - {:?}", new_user_id, temp_reg.name);
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
            value: U32Token::from_u32(17).unwrap() // U32Token::gen_random(), // TODO: revert this!!
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