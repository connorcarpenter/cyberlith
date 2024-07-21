
use auth_server_types::UserId;

pub(crate) struct UserLoginTokenStore {

}

impl UserLoginTokenStore {
    pub fn new() -> Self {
        Self {

        }
    }

    pub fn recv_login_token(&mut self, _user_id: &UserId, _token: &str) {
        panic!("ODST mode does not support this function");
    }

    pub fn spend_login_token(&mut self, token: &str) -> Option<UserId> {
        if token.eq_ignore_ascii_case("odst") {
            Some(UserId::new(1))
        } else {
            None
        }
    }
}