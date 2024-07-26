use std::collections::HashMap;

use bevy_ecs::{
    entity::Entity,
    system::{Commands, Query},
};

use naia_bevy_server::{CommandsExt, RoomKey, Server, UserKey};

use auth_server_types::UserId;
use bevy_http_client::{ApiRequest, ApiResponse, HttpClient, ResponseKey};
use logging::{info, warn};
use session_server_http_proto::SocialLobbyPatch;
use session_server_naia_proto::components::{Lobby, LobbyMember};
use social_server_http_proto::{
    MatchLobbyCreateRequest, MatchLobbyCreateResponse, MatchLobbyJoinRequest,
    MatchLobbyJoinResponse, MatchLobbyLeaveRequest, MatchLobbyLeaveResponse,
    MatchLobbyStartRequest, MatchLobbyStartResponse,
};
use social_server_types::LobbyId;

use crate::{
    session_instance::SessionInstance, social::chat_message_manager::ChatMessageManager,
    user::UserManager,
};

#[derive(PartialEq, Eq, Copy, Clone)]
enum LobbyState {
    WaitingToStart,
    InProgress,
}

enum LobbyReqQueued {
    MatchCreate(UserKey, String),
    MatchJoin(UserKey, LobbyId),
    MatchLeave(UserKey),
    MatchStart(UserKey),
}

enum LobbyReqInFlight {
    MatchCreate(UserId, String, ResponseKey<MatchLobbyCreateResponse>),
    MatchJoin(UserId, LobbyId, ResponseKey<MatchLobbyJoinResponse>),
    MatchLeave(UserId, ResponseKey<MatchLobbyLeaveResponse>),
    MatchStart(UserId, ResponseKey<MatchLobbyStartResponse>),
}

struct LobbyData {
    lobby_owner_user_id: UserId,
    lobby_entity: Entity,
    room_key: RoomKey,
    lobby_member_entities: HashMap<Entity, UserId>,
    state: LobbyState,
}

impl LobbyData {
    fn new(lobby_entity: Entity, room_key: RoomKey, lobby_owner_user_id: UserId) -> Self {
        Self {
            lobby_owner_user_id,
            lobby_entity,
            room_key,
            lobby_member_entities: HashMap::new(),
            state: LobbyState::WaitingToStart,
        }
    }

    pub(crate) fn add_lobby_member_entity(&mut self, lobby_member_entity: Entity, user_id: UserId) {
        self.lobby_member_entities
            .insert(lobby_member_entity, user_id);
    }

    pub(crate) fn remove_lobby_member_entity(&mut self, lobby_member_entity: &Entity) {
        self.lobby_member_entities.remove(lobby_member_entity);
    }

    pub(crate) fn start(&mut self) {
        if self.state != LobbyState::WaitingToStart {
            panic!("Lobby is not waiting to start");
        }
        self.state = LobbyState::InProgress;
    }
}

pub struct LobbyManager {
    queued_requests: Vec<LobbyReqQueued>,
    in_flight_requests: Vec<LobbyReqInFlight>,

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

    pub(crate) fn get_lobby_entity(&self, lobby_id: &LobbyId) -> Option<Entity> {
        self.lobbies
            .get(lobby_id)
            .map(|lobby_data| lobby_data.lobby_entity)
    }

    pub(crate) fn get_lobby_room_key(&self, lobby_id: &LobbyId) -> Option<RoomKey> {
        self.lobbies
            .get(lobby_id)
            .map(|lobby_data| lobby_data.room_key)
    }

    pub(crate) fn update(
        &mut self,
        commands: &mut Commands,
        naia_server: &mut Server,
        http_client: &mut HttpClient,
        user_manager: &mut UserManager,
        lobby_q: &mut Query<&mut Lobby>,
        social_server_url: &Option<(String, u16)>,
        session_instance: &SessionInstance,
        global_room_key: &RoomKey,
    ) {
        self.process_in_flight_requests(
            commands,
            naia_server,
            http_client,
            user_manager,
            lobby_q,
            global_room_key,
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
            match request {
                LobbyReqQueued::MatchCreate(owner_user_key, match_name) => {
                    self.send_match_lobby_create(
                        http_client,
                        user_manager,
                        social_server_url.as_ref(),
                        session_instance,
                        &owner_user_key,
                        &match_name,
                    );
                }
                LobbyReqQueued::MatchJoin(user_key, lobby_id) => {
                    self.send_match_lobby_join(
                        http_client,
                        user_manager,
                        social_server_url.as_ref(),
                        session_instance,
                        &user_key,
                        &lobby_id,
                    );
                }
                LobbyReqQueued::MatchLeave(user_key) => {
                    self.send_match_lobby_leave(
                        http_client,
                        user_manager,
                        social_server_url.as_ref(),
                        session_instance,
                        &user_key,
                    );
                }
                LobbyReqQueued::MatchStart(user_key) => {
                    self.send_match_lobby_start(
                        http_client,
                        user_manager,
                        social_server_url.as_ref(),
                        session_instance,
                        &user_key,
                    );
                }
            }
        }
    }

    fn process_in_flight_requests(
        &mut self,
        commands: &mut Commands,
        naia_server: &mut Server,
        http_client: &mut HttpClient,
        user_manager: &mut UserManager,
        lobby_q: &mut Query<&mut Lobby>,
        main_menu_room_key: &RoomKey,
    ) {
        if self.in_flight_requests.is_empty() {
            // no in-flight requests
            return;
        }

        let mut continuing_requests = Vec::new();
        let in_flight_requests = std::mem::take(&mut self.in_flight_requests);

        for req in in_flight_requests {
            match &req {
                LobbyReqInFlight::MatchCreate(owner_user_id, match_name, response_key) => {
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

                                self.create_lobby(
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
                LobbyReqInFlight::MatchJoin(user_id, lobby_id, response_key) => {
                    if let Some(response_result) = http_client.recv(response_key) {
                        let host = "session";
                        let remote = "social";
                        bevy_http_client::log_util::recv_res(
                            host,
                            remote,
                            MatchLobbyJoinResponse::name(),
                        );

                        match response_result {
                            Ok(_response) => {
                                // info!("received join match lobby message response from social server");

                                self.join_lobby(
                                    commands,
                                    naia_server,
                                    user_manager,
                                    &lobby_id,
                                    user_id,
                                );
                            }
                            Err(e) => {
                                warn!(
                                    "error receiving join match lobby response from social server: {:?}",
                                    e.to_string()
                                );
                            }
                        }
                    } else {
                        continuing_requests.push(req);
                    }
                }
                LobbyReqInFlight::MatchLeave(user_id, response_key) => {
                    if let Some(response_result) = http_client.recv(response_key) {
                        let host = "session";
                        let remote = "social";
                        bevy_http_client::log_util::recv_res(
                            host,
                            remote,
                            MatchLobbyLeaveResponse::name(),
                        );

                        match response_result {
                            Ok(_response) => {
                                // info!("received leave match lobby message response from social server");

                                self.leave_lobby(commands, naia_server, user_manager, user_id);
                            }
                            Err(e) => {
                                warn!(
                                    "error receiving leave match lobby response from social server: {:?}",
                                    e.to_string()
                                );
                            }
                        }
                    } else {
                        continuing_requests.push(req);
                    }
                }
                LobbyReqInFlight::MatchStart(_user_id, response_key) => {
                    if let Some(response_result) = http_client.recv(response_key) {
                        let host = "session";
                        let remote = "social";
                        bevy_http_client::log_util::recv_res(
                            host,
                            remote,
                            MatchLobbyStartResponse::name(),
                        );

                        match response_result {
                            Ok(response) => {
                                // info!("received start match lobby message response from social server");
                                let lobby_id = response.lobby_id();

                                self.start_lobby(lobby_q, &lobby_id);
                            }
                            Err(e) => {
                                warn!(
                                    "error receiving start match lobby response from social server: {:?}",
                                    e.to_string()
                                );
                            }
                        }
                    } else {
                        continuing_requests.push(req);
                    }
                }
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

            self.queued_requests.push(LobbyReqQueued::MatchCreate(
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

        self.in_flight_requests.push(LobbyReqInFlight::MatchCreate(
            owner_user_id,
            match_name.to_string(),
            response_key,
        ));

        return;
    }

    pub(crate) fn send_match_lobby_join(
        &mut self,
        http_client: &mut HttpClient,
        user_manager: &UserManager,
        social_server_url: Option<&(String, u16)>,
        session_instance: &SessionInstance,
        user_key: &UserKey,
        lobby_id: &LobbyId,
    ) {
        let Some(user_id) = user_manager.user_key_to_id(user_key) else {
            warn!("User not found: {:?}", user_key);
            return;
        };

        let Some((social_server_addr, social_server_port)) = social_server_url else {
            warn!("received join match lobby request but no social server is available!");

            self.queued_requests
                .push(LobbyReqQueued::MatchJoin(*user_key, *lobby_id));

            return;
        };

        // info!("sending match lobby join request to social server - [userid {:?}]:(`{:?}`)", sending_user_id, message);
        let request =
            MatchLobbyJoinRequest::new(session_instance.instance_secret(), *lobby_id, user_id);

        let host = "session";
        let remote = "social";
        bevy_http_client::log_util::send_req(host, remote, MatchLobbyJoinRequest::name());
        let response_key = http_client.send(social_server_addr, *social_server_port, request);

        self.in_flight_requests.push(LobbyReqInFlight::MatchJoin(
            user_id,
            *lobby_id,
            response_key,
        ));

        return;
    }

    pub(crate) fn send_match_lobby_start(
        &mut self,
        http_client: &mut HttpClient,
        user_manager: &UserManager,
        social_server_url: Option<&(String, u16)>,
        session_instance: &SessionInstance,
        user_key: &UserKey,
    ) {
        let Some(user_id) = user_manager.user_key_to_id(user_key) else {
            warn!("User not found: {:?}", user_key);
            return;
        };

        let Some((social_server_addr, social_server_port)) = social_server_url else {
            warn!("received start match lobby request but no social server is available!");

            self.queued_requests
                .push(LobbyReqQueued::MatchStart(*user_key));

            return;
        };

        // info!("sending match lobby start request to social server - [userid {:?}]:(`{:?}`)", sending_user_id, message);
        let request = MatchLobbyStartRequest::new(session_instance.instance_secret(), user_id);

        let host = "session";
        let remote = "social";
        bevy_http_client::log_util::send_req(host, remote, MatchLobbyStartRequest::name());
        let response_key = http_client.send(social_server_addr, *social_server_port, request);

        self.in_flight_requests
            .push(LobbyReqInFlight::MatchStart(user_id, response_key));

        return;
    }

    pub(crate) fn send_match_lobby_leave(
        &mut self,
        http_client: &mut HttpClient,
        user_manager: &UserManager,
        social_server_url: Option<&(String, u16)>,
        session_instance: &SessionInstance,
        user_key: &UserKey,
    ) {
        let Some(user_id) = user_manager.user_key_to_id(user_key) else {
            warn!("User not found: {:?}", user_key);
            return;
        };

        let Some((social_server_addr, social_server_port)) = social_server_url else {
            warn!("received leave match lobby request but no social server is available!");

            self.queued_requests
                .push(LobbyReqQueued::MatchLeave(*user_key));

            return;
        };

        // info!("sending match lobby leave request to social server - [userid {:?}]:(`{:?}`)", sending_user_id, message);
        let request = MatchLobbyLeaveRequest::new(session_instance.instance_secret(), user_id);

        let host = "session";
        let remote = "social";
        bevy_http_client::log_util::send_req(host, remote, MatchLobbyLeaveRequest::name());
        let response_key = http_client.send(social_server_addr, *social_server_port, request);

        self.in_flight_requests
            .push(LobbyReqInFlight::MatchLeave(user_id, response_key));

        return;
    }

    pub(crate) fn patch_match_lobbies(
        &mut self,
        commands: &mut Commands,
        naia_server: &mut Server,
        http_client: &mut HttpClient,
        user_manager: &mut UserManager,
        chat_message_manager: &mut ChatMessageManager,
        lobby_q: &mut Query<&mut Lobby>,
        main_menu_room_key: &RoomKey,
        patches: &Vec<SocialLobbyPatch>,
    ) {
        for patch in patches {
            match patch {
                SocialLobbyPatch::Create(lobby_id, match_name, owner_id) => {
                    info!(
                        "creating lobby - [lobbyid {:?}]:(`{:?}`), [ownerid {:?}]",
                        lobby_id, match_name, owner_id
                    );

                    self.create_lobby(
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
                // SocialLobbyPatch::Delete(lobby_id) => {
                //     info!("removing match lobby - [lobbyid {:?}]", lobby_id);
                //
                //     self.remove_lobby(commands, naia_server, lobby_id);
                // }
                SocialLobbyPatch::Join(lobby_id, user_id) => {
                    info!(
                        "joining lobby - [lobbyid {:?}], [userid {:?}]",
                        lobby_id, user_id
                    );

                    self.join_lobby(commands, naia_server, user_manager, lobby_id, user_id);
                }
                SocialLobbyPatch::Leave(user_id) => {
                    info!("leaving lobby - [userid {:?}]", user_id);

                    self.leave_lobby(commands, naia_server, user_manager, user_id);
                }
                SocialLobbyPatch::Message(message_id, timestamp, user_id, message) => {
                    info!(
                        "sending message to lobby - [messageid {:?}], [timestamp {:?}], [userid {:?}], [message {:?}]",
                        message_id, timestamp, user_id, message
                    );

                    chat_message_manager.patch_lobby_chat_message(
                        commands,
                        naia_server,
                        http_client,
                        user_manager,
                        self,
                        main_menu_room_key,
                        message_id,
                        timestamp,
                        user_id,
                        message,
                    );
                }
                SocialLobbyPatch::Start(lobby_id) => {
                    info!("starting lobby match - [lobbyid {:?}]", lobby_id);

                    self.start_lobby(lobby_q, lobby_id);
                }
            }
        }
    }

    pub(crate) fn has_lobby(&self, lobby_id: &LobbyId) -> bool {
        self.lobbies.contains_key(lobby_id)
    }

    pub(crate) fn create_lobby(
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
        self.lobbies.insert(
            *lobby_id,
            LobbyData::new(lobby_entity, lobby_room_key, *owner_user_id),
        );

        let owner_user_entity = user_manager.get_or_init_user_entity(
            commands,
            naia_server,
            http_client,
            main_menu_room_key,
            owner_user_id,
        );

        // set lobby owner
        lobby.owner_user_entity.set(naia_server, &owner_user_entity);
        commands.entity(lobby_entity).insert(lobby);

        // join lobby room
        self.join_lobby(commands, naia_server, user_manager, lobby_id, owner_user_id);
    }

    fn remove_lobby(
        &mut self,
        commands: &mut Commands,
        naia_server: &mut Server,
        user_manager: &mut UserManager,
        lobby_id: &LobbyId,
    ) {
        if let Some(lobby_data) = self.lobbies.remove(lobby_id) {
            let LobbyData {
                lobby_entity,
                room_key,
                lobby_member_entities,
                ..
            } = lobby_data;

            // despawn lobby entity
            commands.entity(lobby_entity).despawn();
            // remove room
            naia_server.room_mut(&room_key).destroy();

            // despawn lobby member entities, and remove link to user
            for (lobby_member_entity, user_id) in lobby_member_entities {
                // despawn lobby member entity
                commands.entity(lobby_member_entity).despawn();

                // remove link to user
                user_manager.user_leave_lobby(&user_id);
            }
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
        user_manager: &mut UserManager,
        lobby_id: &LobbyId,
        joining_user_id: &UserId,
    ) {
        // create LobbyMember entity
        let lobby_member_entity = commands.spawn_empty().enable_replication(naia_server).id();

        let lobby_data = self.lobbies.get_mut(lobby_id).unwrap();

        // add lobby member entity to collection
        lobby_data.add_lobby_member_entity(lobby_member_entity, *joining_user_id);

        // get lobby room key & entity
        let lobby_room_key = &lobby_data.room_key;
        let lobby_entity = &lobby_data.lobby_entity;

        // get user key & entity
        let (joining_user_key, joining_user_entity) =
            user_manager.user_join_lobby(joining_user_id, lobby_id, &lobby_member_entity);

        // add user and lobbymember to room
        naia_server
            .room_mut(lobby_room_key)
            // add user to lobby room
            .add_user(&joining_user_key)
            // add LobbyMember entity to lobby room
            .add_entity(&lobby_member_entity);

        // create & setup LobbyMember component
        let mut lobby_member = LobbyMember::new();
        lobby_member.lobby_entity.set(naia_server, lobby_entity);
        lobby_member
            .user_entity
            .set(naia_server, &joining_user_entity);
        commands.entity(lobby_member_entity).insert(lobby_member);
    }

    fn leave_lobby(
        &mut self,
        commands: &mut Commands,
        naia_server: &mut Server,
        user_manager: &mut UserManager,
        leaving_user_id: &UserId,
    ) {
        // get user key & entity & lobby_id
        let (lobby_id, lobby_member_entity) = user_manager.user_leave_lobby(leaving_user_id);

        // despawn lobby_member entity
        commands.entity(lobby_member_entity).despawn();
        {
            let lobby_data = self.lobbies.get_mut(&lobby_id).unwrap();
            lobby_data.remove_lobby_member_entity(&lobby_member_entity);
        }

        // determine if this is the owner
        let lobby_data = self.lobbies.get(&lobby_id).unwrap();
        let lobby_owner_user_id = lobby_data.lobby_owner_user_id;
        if lobby_owner_user_id == *leaving_user_id {
            // delete the lobby
            self.remove_lobby(commands, naia_server, user_manager, &lobby_id);
        }
    }

    fn start_lobby(&mut self, lobby_q: &mut Query<&mut Lobby>, lobby_id: &LobbyId) {
        let lobby_data = self.lobbies.get_mut(&lobby_id).unwrap();
        lobby_data.start();
        let lobby_entity = lobby_data.lobby_entity;
        let mut lobby = lobby_q.get_mut(lobby_entity).unwrap();
        lobby.start();
    }
}
