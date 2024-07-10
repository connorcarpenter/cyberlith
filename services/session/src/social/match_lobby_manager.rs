use std::collections::HashMap;

use bevy_ecs::{
    entity::Entity,
    system::Commands,
};

use naia_bevy_server::{CommandsExt, RoomKey, Server, UserKey};

use bevy_http_client::{ApiRequest, ApiResponse, HttpClient, ResponseKey};
use logging::{info, warn};

use auth_server_types::UserId;
use session_server_http_proto::SocialLobbyPatch;
use session_server_naia_proto::components::MatchLobby;

use social_server_http_proto::{MatchLobbyCreateResponse, MatchLobbyCreateRequest};
use social_server_types::MatchLobbyId;

use crate::{user::UserManager, session_instance::SessionInstance};

struct MatchCreateReqQueued(UserKey, String);
struct MatchCreateReqInFlight(UserId, String, ResponseKey<MatchLobbyCreateResponse>);

pub struct MatchLobbyManager {

    queued_requests: Vec<MatchCreateReqQueued>,
    in_flight_requests: Vec<MatchCreateReqInFlight>,

    match_lobbies_room_key: Option<RoomKey>,
    match_lobbies: HashMap<MatchLobbyId, Entity>,
}

impl MatchLobbyManager {
    pub fn new() -> Self {
        Self {

            queued_requests: Vec::new(),
            in_flight_requests: Vec::new(),

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
        commands: &mut Commands,
        naia_server: &mut Server,
        http_client: &mut HttpClient,
        user_manager: &mut UserManager,
        social_server_url: &Option<(String, u16)>,
        session_instance: &SessionInstance,
        user_presence_room_key: &RoomKey,
    ) {
        self.process_in_flight_requests(
            commands,
            naia_server,
            http_client,
            user_manager,
            user_presence_room_key,
        );
        self.process_queued_requests(
            http_client,
            social_server_url,
            session_instance,
            user_manager,
        );
    }

    pub fn room_key(&self) -> RoomKey {
        self.match_lobbies_room_key.unwrap()
    }

    fn process_queued_requests(
        &mut self,
        http_client: &mut HttpClient,
        social_server_url: &Option<(String, u16)>,
        session_instance: &SessionInstance,
        user_manager: &UserManager,
    ) {
        if self.queued_requests.is_empty() {
            // no queued requests
            return;
        }
        if social_server_url.is_none() {
            // it's okay to wait until the social server is available
            return;
        };

        let queued_requests = std::mem::take(&mut self.queued_requests);
        for request in queued_requests {
            self.send_match_lobby_create(
                http_client,
                user_manager,
                social_server_url.as_ref(),
                session_instance,
                &request.0,
                &request.1,
            );
        }
    }

    fn process_in_flight_requests(
        &mut self,
        commands: &mut Commands,
        naia_server: &mut Server,
        http_client: &mut HttpClient,
        user_manager: &mut UserManager,
        user_presence_room_key: &RoomKey,
    ) {
        if self.in_flight_requests.is_empty() {
            // no in-flight requests
            return;
        }

        let mut continuing_requests = Vec::new();
        let in_flight_requests = std::mem::take(&mut self.in_flight_requests);

        for req in in_flight_requests {

            let MatchCreateReqInFlight(owner_user_id, match_name, response_key) = &req;

            if let Some(response_result) = http_client.recv(response_key) {
                let host = "session";
                let remote = "social";
                bevy_http_client::log_util::recv_res(
                    host,
                    remote,
                    MatchLobbyCreateResponse::name(),
                );

                match response_result {
                    Ok(response) => {
                        // info!("received create match lobby message response from social server");
                        let lobby_id = response.match_lobby_id();

                        self.add_match_lobby(
                            commands,
                            naia_server,
                            http_client,
                            user_manager,
                            user_presence_room_key,
                            &lobby_id,
                            match_name,
                            owner_user_id,
                        );
                    }
                    Err(e) => {
                        warn!("error receiving create match lobby response from social server: {:?}", e.to_string());
                    }
                }
            } else {
                continuing_requests.push(req);
            }
        }

        self.in_flight_requests = continuing_requests;
    }

    pub(crate) fn send_match_lobby_create(
        &mut self,
        http_client: &mut HttpClient,
        user_manager: &UserManager,
        social_server_url: Option<&(String, u16)>,
        session_instance: &SessionInstance,
        owner_user_key: &UserKey,
        match_name: &str,
    ) {
        let Some(owner_user_id) = user_manager.user_key_to_id(owner_user_key) else {
            warn!("User not found: {:?}", owner_user_key);
            return;
        };

        let Some((social_server_addr, social_server_port)) = social_server_url else {
            warn!("received create match lobby request but no social server is available!");

            self.queued_requests.push(MatchCreateReqQueued(*owner_user_key, match_name.to_string()));

            return;
        };

        // info!("sending global chat send message request to social server - [userid {:?}]:(`{:?}`)", sending_user_id, message);
        let request = MatchLobbyCreateRequest::new(
            session_instance.instance_secret(),
            owner_user_id,
            match_name,
        );

        let host = "session";
        let remote = "social";
        bevy_http_client::log_util::send_req(host, remote, MatchLobbyCreateRequest::name());
        let response_key = http_client.send(social_server_addr, *social_server_port, request);

        self.in_flight_requests.push(MatchCreateReqInFlight(owner_user_id, match_name.to_string(), response_key));

        return;
    }

    pub(crate) fn patch_match_lobbies(
        &mut self,
        commands: &mut Commands,
        naia_server: &mut Server,
        http_client: &mut HttpClient,
        user_manager: &mut UserManager,
        user_presence_room_key: &RoomKey,
        patches: &Vec<SocialLobbyPatch>,
    ) {
        for patch in patches {
            match patch {
                SocialLobbyPatch::Create(lobby_id, match_name, owner_id) => {
                    info!("adding match lobby - [lobbyid {:?}]:(`{:?}`), [ownerid {:?}]", lobby_id, match_name, owner_id);

                    self.add_match_lobby(commands, naia_server, http_client, user_manager, user_presence_room_key, lobby_id, match_name, owner_id);
                }
                SocialLobbyPatch::Delete(lobby_id) => {
                    info!("removing match lobby - [lobbyid {:?}]", lobby_id);

                    self.remove_match_lobby(commands, lobby_id);
                }
            }
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
