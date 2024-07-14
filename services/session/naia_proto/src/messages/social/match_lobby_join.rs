use naia_bevy_shared::Message;

use social_server_types::LobbyId;

#[derive(Message)]
pub struct MatchLobbyJoin {
    pub match_id: LobbyId,
}

impl MatchLobbyJoin {
    pub fn new(match_id: LobbyId) -> Self {
        Self { match_id }
    }
}
