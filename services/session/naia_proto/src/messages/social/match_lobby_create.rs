use naia_bevy_shared::Message;

#[derive(Message)]
pub struct MatchLobbyCreate {
    pub match_name: String,
}

impl MatchLobbyCreate {
    pub fn new(match_name: &str) -> Self {
        Self {
            match_name: match_name.to_string(),
        }
    }
}
