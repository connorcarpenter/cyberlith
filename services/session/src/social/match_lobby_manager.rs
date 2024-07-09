use std::collections::HashMap;

use bevy_ecs::{
    entity::Entity,
    system::Commands,
};

use naia_bevy_server::{CommandsExt, RoomKey, Server};

use bevy_http_client::HttpClient;
use logging::{info, warn};

use auth_server_types::UserId;
use session_server_naia_proto::components::MatchLobby;
use social_server_types::MatchLobbyId;

use crate::user::UserManager;

pub struct MatchLobbyManager {

    match_lobbies_room_key: Option<RoomKey>,
    match_lobbies: HashMap<MatchLobbyId, Entity>,
}

impl MatchLobbyManager {
    pub fn new() -> Self {
        Self {

            match_lobbies_room_key: None,
            match_lobbies: HashMap::new(),
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

    pub fn room_key(&self) -> RoomKey {
        self.match_lobbies_room_key.unwrap()
    }

    pub(crate) fn patch_match_lobbies(
        &mut self,
        commands: &mut Commands,
        naia_server: &mut Server,
        http_client: &mut HttpClient,
        user_manager: &mut UserManager,
        user_presence_room_key: &RoomKey,
        added_lobbies: &Vec<(MatchLobbyId, String, UserId)>,
        removed_lobbies: &Vec<MatchLobbyId>,
    ) {
        for (lobby_id, lobby_name, owner_id) in added_lobbies {
            info!("adding match lobby - [lobbyid {:?}]:(`{:?}`), [ownerid {:?}]", lobby_id, lobby_name, owner_id);

            self.add_match_lobby(commands, naia_server, http_client, user_manager, user_presence_room_key, lobby_id, lobby_name, owner_id);
        }

        for lobby_id in removed_lobbies {
            info!("removing match lobby - [lobbyid {:?}]", lobby_id);

            self.remove_match_lobby(commands, lobby_id);
        }
    }

    fn add_match_lobby(
        &mut self,
        commands: &mut Commands,
        naia_server: &mut Server,
        http_client: &mut HttpClient,
        user_manager: &mut UserManager,
        user_presence_room_key: &RoomKey,
        lobby_id: &MatchLobbyId,
        lobby_name: &str,
        owner_user_id: &UserId
    ) {
        // spawn lobby entity
        let match_lobby_entity = commands
            .spawn_empty()
            .enable_replication(naia_server)
            .id();
        let mut match_lobby = MatchLobby::new(
            *lobby_id,
            lobby_name,
        );

        // add to lobbies room
        naia_server
            .room_mut(&self.room_key())
            .add_entity(&match_lobby_entity);

        // add to collection
        self.match_lobbies.insert(*lobby_id, match_lobby_entity);

        let owner_user_entity = {
            if let Some(user_entity) = user_manager.get_user_entity(owner_user_id) {
                user_entity
            } else {
                user_manager.add_user_data(commands, naia_server, http_client, user_presence_room_key, owner_user_id);

                let user_entity = user_manager.get_user_entity(owner_user_id).unwrap();
                user_entity
            }
        };

        match_lobby.user_entity.set(naia_server, &owner_user_entity);
        commands
            .entity(match_lobby_entity)
            .insert(match_lobby);
    }

    fn remove_match_lobby(
        &mut self,
        commands: &mut Commands,
        lobby_id: &MatchLobbyId
    ) {
        if let Some(removed_entity) = self.match_lobbies.remove(lobby_id) {
            commands.entity(removed_entity).despawn();
        } else {
            warn!("attempted to remove non-existent match lobby - [lobbyid {:?}]", lobby_id);
        }
    }
}
