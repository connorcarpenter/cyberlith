use std::time::Duration;

use crate::{
    global_chat::GlobalChatState, match_lobbies::MatchLobbiesState, region::RegionServerState,
    session_servers::SessionServersState, users::UsersState,
};

pub struct State {
    pub region_server: RegionServerState,
    pub session_servers: SessionServersState,
    pub match_lobbies: MatchLobbiesState,
    pub users: UsersState,
    pub global_chat: GlobalChatState,
}

impl State {
    pub fn new(
        registration_resend_rate: Duration,
        region_server_disconnect_timeout: Duration,
    ) -> Self {
        Self {
            region_server: RegionServerState::new(
                registration_resend_rate,
                region_server_disconnect_timeout,
            ),
            session_servers: SessionServersState::new(),
            match_lobbies: MatchLobbiesState::new(),
            users: UsersState::new(),
            global_chat: GlobalChatState::new(),
        }
    }
}
