use std::collections::{HashMap, HashSet, VecDeque};

use auth_server_types::UserId;
use social_server_types::{LobbyId, MessageId, Timestamp};

use crate::session_servers::SessionServerId;

#[derive(Eq, PartialEq, Copy, Clone)]
pub(crate) enum LobbyState {
    WaitingToStart,
    InProgress,
}

pub(crate) enum LobbyPatch {
    Create(LobbyId, UserId, String),
    Join(LobbyId, UserId),
    Leave(UserId),
    Message(MessageId, Timestamp, UserId, String),
    Start(LobbyId),
}

struct LobbyData {
    owner_user_id: UserId,
    match_name: String,
    users: HashSet<UserId>,
    message_log: VecDeque<(MessageId, Timestamp, UserId, String)>,
    next_message_id: MessageId,
    state: LobbyState,
}

impl LobbyData {
    pub fn new(owner_user_id: UserId, match_name: String) -> Self {
        let mut users = HashSet::new();
        users.insert(owner_user_id);
        Self {
            owner_user_id,
            match_name,
            users,
            message_log: VecDeque::new(),
            next_message_id: MessageId::new(0),
            state: LobbyState::WaitingToStart,
        }
    }

    pub fn send_message(&mut self, user_id: &UserId, message: &str) -> (MessageId, Timestamp) {
        // get next lobby chat id
        let next_message_id = self.next_message_id;
        self.next_message_id = self.next_message_id.next();

        // get timestamp
        let timestamp = Timestamp::now();

        // add to message log
        self.message_log
            .push_back((next_message_id, timestamp, *user_id, message.to_string()));
        if self.message_log.len() > 100 {
            self.message_log.pop_front();
        }

        (next_message_id, timestamp)
    }
}

pub struct MatchLobbiesState {
    lobbies: HashMap<LobbyId, LobbyData>,
    next_lobby_id: LobbyId,

    starting_lobbies: Vec<LobbyId>,

    // the session server id here is the SENDER not the RECEIVER
    outgoing_patches: HashMap<SessionServerId, Vec<LobbyPatch>>,
}

impl MatchLobbiesState {
    pub fn new() -> Self {
        Self {
            lobbies: HashMap::new(),
            next_lobby_id: LobbyId::new(0),

            starting_lobbies: Vec::new(),

            outgoing_patches: HashMap::new(),
        }
    }

    pub fn create(
        &mut self,
        session_instance_id: &SessionServerId,
        match_name: &str,
        creator_user_id: &UserId,
    ) -> LobbyId {
        let new_lobby_id = self.next_lobby_id;
        self.next_lobby_id = self.next_lobby_id.next();

        self.lobbies
            .insert(new_lobby_id, LobbyData::new(*creator_user_id, match_name.to_string()));

        // add to outgoing patches
        if !self.outgoing_patches.contains_key(&session_instance_id) {
            self.outgoing_patches
                .insert(*session_instance_id, Vec::new());
        }
        let session_server_patches = self.outgoing_patches.get_mut(session_instance_id).unwrap();
        session_server_patches.push(LobbyPatch::Create(
            new_lobby_id,
            *creator_user_id,
            match_name.to_string(),
        ));

        new_lobby_id
    }

    pub fn join(
        &mut self,
        session_server_id: &SessionServerId,
        lobby_id: &LobbyId,
        joining_user_id: &UserId,
    ) {
        let lobby_data = self.lobbies.get_mut(lobby_id).unwrap();
        lobby_data.users.insert(*joining_user_id);

        // add to outgoing patches
        if !self
            .outgoing_patches
            .contains_key(&session_server_id)
        {
            self.outgoing_patches
                .insert(*session_server_id, Vec::new());
        }
        let session_server_patches = self
            .outgoing_patches
            .get_mut(&session_server_id)
            .unwrap();
        session_server_patches.push(LobbyPatch::Join(*lobby_id, *joining_user_id));
    }

    pub fn leave(
        &mut self,
        session_server_id: &SessionServerId,
        lobby_id: &LobbyId,
        leaving_user_id: &UserId
    ) {
        let lobby_data = self.lobbies.get_mut(lobby_id).unwrap();
        lobby_data.users.remove(leaving_user_id);

        let owner_user_id = lobby_data.owner_user_id;
        if owner_user_id == *leaving_user_id {
            // delete the lobby
            self.lobbies.remove(lobby_id);
        }

        // add to outgoing patches
        if !self
            .outgoing_patches
            .contains_key(&session_server_id)
        {
            self.outgoing_patches
                .insert(*session_server_id, Vec::new());
        }

        let session_server_patches = self
            .outgoing_patches
            .get_mut(&session_server_id)
            .unwrap();
        session_server_patches.push(LobbyPatch::Leave(*leaving_user_id));
    }

    pub fn send_message(
        &mut self,
        sending_session_server_id: &SessionServerId,
        lobby_id: &LobbyId,
        user_id: &UserId,
        message: &str,
    ) -> (MessageId, Timestamp) {

        let lobby_data = self.lobbies.get_mut(&lobby_id).unwrap();
        let (msg_id, timestamp) = lobby_data.send_message(user_id, message);

        // add to outgoing patches
        if !self
            .outgoing_patches
            .contains_key(sending_session_server_id)
        {
            self.outgoing_patches
                .insert(*sending_session_server_id, Vec::new());
        }
        let session_server_patches = self
            .outgoing_patches
            .get_mut(sending_session_server_id)
            .unwrap();
        session_server_patches.push(LobbyPatch::Message(msg_id, timestamp, *user_id, message.to_string()));

        (msg_id, timestamp)
    }

    pub fn start(
        &mut self,
        session_server_id: &SessionServerId,
        lobby_id: &LobbyId,
        starting_user_id: &UserId
    ) -> Result<(), String> {
        let lobby_data = self.lobbies.get_mut(lobby_id).unwrap();
        if lobby_data.owner_user_id != *starting_user_id {
            return Err("user is not the owner of the lobby".to_string());
        }
        if lobby_data.state != LobbyState::WaitingToStart {
            return Err("lobby is not waiting to start".to_string());
        }
        lobby_data.state = LobbyState::InProgress;

        self.starting_lobbies.push(*lobby_id);

        // add to outgoing patches
        if !self
            .outgoing_patches
            .contains_key(&session_server_id)
        {
            self.outgoing_patches
                .insert(*session_server_id, Vec::new());
        }

        let session_server_patches = self
            .outgoing_patches
            .get_mut(&session_server_id)
            .unwrap();
        session_server_patches.push(LobbyPatch::Start(*lobby_id));

        return Ok(());
    }

    pub fn get_lobbies(&self) -> Vec<(LobbyId, UserId, String, Vec<UserId>, LobbyState)> {
        let mut output = Vec::new();

        for (lobby_id, lobby_data) in self.lobbies.iter() {
            let owner_user_id = lobby_data.owner_user_id;
            let match_name = lobby_data.match_name.clone();
            let users = lobby_data.users.iter().map(|x| *x).collect();
            let state = lobby_data.state.clone();
            output.push((*lobby_id, owner_user_id, match_name, users, state));
        }

        output
    }

    pub fn get_lobby_users(&self, lobby_id: &LobbyId) -> Vec<UserId> {
        let lobby_data = self.lobbies.get(lobby_id).unwrap();
        lobby_data.users.iter().map(|x| *x).collect()
    }

    pub fn take_patches(&mut self) -> HashMap<SessionServerId, Vec<LobbyPatch>> {
        std::mem::take(&mut self.outgoing_patches)
    }

    pub fn take_starting_lobbies(&mut self) -> Vec<LobbyId> {
        std::mem::take(&mut self.starting_lobbies)
    }
}
