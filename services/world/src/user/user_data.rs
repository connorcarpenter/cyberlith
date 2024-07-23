use auth_server_types::UserId;
use social_server_types::LobbyId;

pub struct UserData {
    pub session_server_addr: String,
    pub session_server_port: u16,
    pub user_id: UserId,
    pub lobby_id: LobbyId,

}

impl UserData {
    pub(crate) fn new(
        session_server_addr: &str,
        session_server_port: u16,
        user_id: UserId,
        lobby_id: LobbyId
    ) -> Self {
        Self {
            session_server_addr: session_server_addr.to_string(),
            session_server_port,
            user_id,
            lobby_id,
        }
    }
}