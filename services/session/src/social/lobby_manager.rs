use std::collections::HashMap;

use bevy_ecs::{entity::Entity, system::Commands};

use naia_bevy_server::{CommandsExt, RoomKey, Server, UserKey};

use bevy_http_client::{ApiRequest, ApiResponse, HttpClient, ResponseKey};
use logging::{info, warn};

use auth_server_types::UserId;

use session_server_http_proto::SocialLobbyPatch;
use session_server_naia_proto::components::{Lobby, LobbyMember};

use social_server_http_proto::{MatchLobbyCreateRequest, MatchLobbyCreateResponse};
use social_server_types::LobbyId;

use crate::{session_instance::SessionInstance, user::UserManager};

struct MatchCreateReqQueued(UserKey, String);
struct MatchCreateReqInFlight(UserId, String, ResponseKey<MatchLobbyCreateResponse>);

struct LobbyData {
    lobby_entity: Entity,
    room_key: RoomKey,
}

impl LobbyData {
    fn new(lobby_entity: Entity, room_key: RoomKey) -> Self {
        Self {
            lobby_entity,
            room_key,
        }
    }

}

pub struct LobbyManager {
    queued_requests: Vec<MatchCreateReqQueued>,
    in_flight_requests: Vec<MatchCreateReqInFlight>,

    lobbies: HashMap<LobbyId, LobbyData>,
}

impl LobbyManager {
    pub fn new() -> Self {
        Self {
            queued_requests: Vec::new(),
            in_flight_requests: Vec::new(),

            lobbies: HashMap::new(),
        }
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
        main_menu_room_key: &RoomKey,
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

                        self.add_lobby(
                            commands,
                            naia_server,
                            http_client,
                            user_manager,
                            main_menu_room_key,
                            &lobby_id,
                            match_name,
                            owner_user_id,
                        );
                    }
                    Err(e) => {
                        warn!(
                            "error receiving create match lobby response from social server: {:?}",
                            e.to_string()
                        );
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

            self.queued_requests.push(MatchCreateReqQueued(
                *owner_user_key,
                match_name.to_string(),
            ));

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

        self.in_flight_requests.push(MatchCreateReqInFlight(
            owner_user_id,
            match_name.to_string(),
            response_key,
        ));

        return;
    }

    pub(crate) fn patch_match_lobbies(
        &mut self,
        commands: &mut Commands,
        naia_server: &mut Server,
        http_client: &mut HttpClient,
        user_manager: &mut UserManager,
        main_menu_room_key: &RoomKey,
        patches: &Vec<SocialLobbyPatch>,
    ) {
        for patch in patches {
            match patch {
                SocialLobbyPatch::Create(lobby_id, match_name, owner_id) => {
                    info!(
                        "adding match lobby - [lobbyid {:?}]:(`{:?}`), [ownerid {:?}]",
                        lobby_id, match_name, owner_id
                    );

                    self.add_lobby(
                        commands,
                        naia_server,
                        http_client,
                        user_manager,
                        main_menu_room_key,
                        lobby_id,
                        match_name,
                        owner_id,
                    );
                }
                SocialLobbyPatch::Delete(lobby_id) => {
                    info!("removing match lobby - [lobbyid {:?}]", lobby_id);

                    self.remove_lobby(commands, naia_server, lobby_id);
                }
            }
        }
    }

    fn add_lobby(
        &mut self,
        commands: &mut Commands,
        naia_server: &mut Server,
        http_client: &mut HttpClient,
        user_manager: &mut UserManager,
        main_menu_room_key: &RoomKey,
        lobby_id: &LobbyId,
        lobby_name: &str,
        owner_user_id: &UserId,
    ) {
        // spawn lobby entity
        let lobby_entity = commands.spawn_empty().enable_replication(naia_server).id();
        let mut lobby = Lobby::new(*lobby_id, lobby_name);

        // add to main menu room
        naia_server
            .room_mut(main_menu_room_key)
            .add_entity(&lobby_entity);

        let lobby_room_key = naia_server.make_room().key();

        // add to collection
        self.lobbies.insert(*lobby_id, LobbyData::new(lobby_entity, lobby_room_key));

        let owner_user_entity = {
            if let Some(user_entity) = user_manager.get_user_entity(owner_user_id) {
                user_entity
            } else {
                user_manager.add_user_data(
                    commands,
                    naia_server,
                    http_client,
                    main_menu_room_key,
                    owner_user_id,
                );

                let user_entity = user_manager.get_user_entity(owner_user_id).unwrap();
                user_entity
            }
        };

        // set lobby owner
        lobby
            .owner_user_entity
            .set(naia_server, &owner_user_entity);
        commands.entity(lobby_entity).insert(lobby);

        // join lobby room
        self.join_lobby(naia_server, lobby_id, owner_user_id);
    }

    fn remove_lobby(&mut self, commands: &mut Commands, naia_server: &mut Server, lobby_id: &LobbyId) {
        if let Some(lobby_data) = self.lobbies.remove(lobby_id) {
            let LobbyData { lobby_entity, room_key } = lobby_data;
            // despawn entity
            commands.entity(lobby_entity).despawn();
            // remove room
            naia_server.room_mut(&room_key).destroy();
        } else {
            warn!(
                "attempted to remove non-existent match lobby - [lobbyid {:?}]",
                lobby_id
            );
        }
    }

    fn join_lobby(
        &mut self,
        commands: &mut Commands,
        naia_server: &mut Server,
        user_manager: &UserManager,
        lobby_id: &LobbyId,
        joining_user_id: &UserId
    ) {
        // get lobby room key & entity
        let lobby = self.lobbies.get(lobby_id).unwrap();
        let lobby_room_key = &lobby.room_key;
        let lobby_entity = &lobby.lobby_entity;

        // get user key & entity
        let joining_user_key = user_manager.user_id_to_key(joining_user_id).unwrap();
        let joining_user_entity = user_manager.get_user_entity(joining_user_id).unwrap();

        // create LobbyMember entity
        let lobby_member_id = commands.spawn_empty().enable_replication(naia_server).id();

        // add user and lobbymember to room
        naia_server
            .room_mut(lobby_room_key)
            // add user to lobby room
            .add_user(&joining_user_key)
            // add LobbyMember entity to lobby room
            .add_entity(&lobby_member_id);

        // create & setup LobbyMember component
        let mut lobby_member = LobbyMember::new();
        lobby_member.lobby_entity.set(naia_server, lobby_entity);
        lobby_member.user_entity.set(naia_server, &joining_user_entity);
        commands.entity(lobby_member_id).insert(lobby_member);
    }
}
