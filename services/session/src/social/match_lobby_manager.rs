use std::collections::VecDeque;

use bevy_ecs::{
    entity::Entity,
    system::Commands,
};

use naia_bevy_server::{RoomKey, Server};

use bevy_http_client::HttpClient;
use logging::info;

use auth_server_types::UserId;
use social_server_types::MatchLobbyId;

use crate::user::UserManager;

pub struct MatchLobbyManager {

    match_lobbies_room_key: Option<RoomKey>,
    match_lobbies: VecDeque<Entity>,
}

impl MatchLobbyManager {
    pub fn new() -> Self {
        Self {

            match_lobbies_room_key: None,
            match_lobbies: VecDeque::new(),
        }
    }

    pub(crate) fn startup(&mut self, naia_server: &mut Server) {
        let match_lobbies_room_key = naia_server.make_room().key();
        self.match_lobbies_room_key = Some(match_lobbies_room_key);
    }

    pub(crate) fn update(
        &mut self,
    ) {
        self.process_in_flight_requests();
        self.process_queued_requests();
    }

    fn process_queued_requests(&mut self) {
        // TODO?
    }

    fn process_in_flight_requests(&mut self) {
        // TODO?
    }

    pub fn get_match_lobbies_room_key(&self) -> RoomKey {
        self.match_lobbies_room_key.unwrap()
    }

    pub(crate) fn patch_match_lobbies(
        &mut self,
        commands: &mut Commands,
        naia_server: &mut Server,
        http_client: &mut HttpClient,
        user_manager: &mut UserManager,
        added_lobbies: &Vec<(MatchLobbyId, String, UserId)>,
        removed_lobbies: &Vec<MatchLobbyId>,
    ) {
        for (lobby_id, lobby_name, owner_id) in added_lobbies {
            info!("adding match lobby - [lobbyid {:?}]:(`{:?}`), [ownerid {:?}]", lobby_id, lobby_name, owner_id);

            self.add_match_lobby(lobby_id, lobby_name, owner_id);
        }

        for lobby_id in removed_lobbies {
            info!("removing match lobby - [lobbyid {:?}]", lobby_id);

            self.remove_match_lobby(lobby_id);
        }
    }

    fn add_match_lobby(&mut self, lobby_id: &MatchLobbyId, lobby_name: &str, owner_id: &UserId) {
        todo!()
    }

    fn remove_match_lobby(&mut self, lobby_id: &MatchLobbyId) {
        todo!()
    }
}
