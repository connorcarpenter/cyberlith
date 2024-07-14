use std::collections::HashMap;

use auth_server_types::UserId;

use social_server_types::MatchLobbyId;

use crate::session_servers::SessionServerId;

pub(crate) enum LobbyPatch {
    Create(MatchLobbyId, UserId, String),
    Delete(MatchLobbyId),
}

pub struct MatchLobbiesState {
    // lobby id, user_id, match_name
    lobbies: HashMap<MatchLobbyId, (UserId, String)>,
    next_lobby_id: MatchLobbyId,

    // the session server id here is the SENDER not the RECEIVER
    outgoing_patches: HashMap<SessionServerId, Vec<LobbyPatch>>,
}

impl MatchLobbiesState {
    pub fn new() -> Self {
        Self {
            lobbies: HashMap::new(),
            next_lobby_id: MatchLobbyId::new(0),

            outgoing_patches: HashMap::new(),
        }
    }

    pub fn create(
        &mut self,
        session_instance_id: SessionServerId,
        match_name: &str,
        creator_user_id: UserId,
    ) -> MatchLobbyId {
        let new_lobby_id = self.next_lobby_id;
        self.next_lobby_id = self.next_lobby_id.next();

        self.lobbies
            .insert(new_lobby_id, (creator_user_id, match_name.to_string()));

        if !self.outgoing_patches.contains_key(&session_instance_id) {
            self.outgoing_patches
                .insert(session_instance_id, Vec::new());
        }
        let session_server_patches = self.outgoing_patches.get_mut(&session_instance_id).unwrap();
        session_server_patches.push(LobbyPatch::Create(
            new_lobby_id,
            creator_user_id,
            match_name.to_string(),
        ));

        new_lobby_id
    }

    pub fn join(
        &mut self,
        session_server_id: SessionServerId,
        match_lobby_id: MatchLobbyId,
        joining_user_id: UserId,
    ) {
        // TODO
    }

    pub fn leave(&mut self, session_server_id: SessionServerId, leaving_user_id: UserId) {
        // TODO
    }

    pub fn send_message(
        &mut self,
        session_server_id: SessionServerId,
        user_id: UserId,
        message: &str,
    ) {
        // TODO
    }

    pub fn get_lobbies(&self) -> &HashMap<MatchLobbyId, (UserId, String)> {
        &self.lobbies
    }

    pub fn take_patches(&mut self) -> HashMap<SessionServerId, Vec<LobbyPatch>> {
        std::mem::take(&mut self.outgoing_patches)
    }
}
