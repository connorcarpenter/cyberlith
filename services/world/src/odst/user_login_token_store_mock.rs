use auth_server_types::UserId;
use social_server_types::LobbyId;

use crate::user::user_data::UserData;

pub(crate) struct UserLoginTokenStore {}

impl UserLoginTokenStore {
    pub fn new() -> Self {
        Self {}
    }

    pub fn clear(&mut self) {
        //
    }

    pub fn recv_login_token(
        &mut self,
        _lobby_id: &LobbyId,
        _login_tokens: &Vec<(String, u16, Vec<(UserId, String)>)>,
    ) {
        panic!("ODST mode does not support this method");
    }

    pub fn spend_login_token(&mut self, token: &str) -> Option<UserData> {
        let tokens: Vec<String> = token.split(":").map(|s| s.to_string()).collect();
        if tokens[0].eq_ignore_ascii_case("odst") {
            let user_id_u64: u64 = tokens[1].parse().unwrap();
            // info!("ODST mode spend_login_token user_id_u64: {}", user_id_u64);
            let user_id = UserId::new(user_id_u64);
            let lobby_id = LobbyId::new(1);
            let session_addr = config::SESSION_SERVER_RECV_ADDR.to_string();
            let session_port = config::SESSION_SERVER_HTTP_PORT;

            Some(UserData::new(
                &session_addr,
                session_port,
                user_id,
                lobby_id,
            ))
        } else {
            None
        }
    }
}
