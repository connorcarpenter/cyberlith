use std::{
    collections::{HashMap, HashSet},
    time::{Duration, Instant},
};

use bevy_ecs::{
    change_detection::ResMut,
    entity::Entity,
    prelude::{Commands, Query},
    system::Resource,
};

use naia_bevy_server::{CommandsExt, RoomKey, Server, UserKey};

use bevy_http_client::{ApiRequest, ApiResponse, HttpClient, ResponseKey};
use config::{AUTH_SERVER_PORT, AUTH_SERVER_RECV_ADDR};
use logging::{info, warn};

use auth_server_http_proto::{UserGetRequest, UserGetResponse};
use auth_server_types::UserId;

use session_server_naia_proto::components::User;

use social_server_types::LobbyId;

use crate::user::user_data::UserData;

#[derive(Resource)]
pub struct UserManager {
    login_tokens: HashMap<String, UserId>,
    user_key_to_id: HashMap<UserKey, UserId>,
    user_data: HashMap<UserId, UserData>,
    inflight_user_info_requests: HashSet<UserId>,
}

impl UserManager {
    pub fn new() -> Self {
        Self {
            login_tokens: HashMap::new(),
            user_key_to_id: HashMap::new(),
            user_data: HashMap::new(),
            inflight_user_info_requests: HashSet::new(),
        }
    }

    // used as a system
    pub fn update(
        mut commands: Commands,
        mut http_client: ResMut<HttpClient>,
        mut user_manager: ResMut<Self>,
    ) {
        user_manager.process_in_flight_requests(&mut commands, &mut http_client);
    }

    // Client login

    pub fn add_login_token(&mut self, user_id: &UserId, token: &str) {
        self.login_tokens.insert(token.to_string(), user_id.clone());
    }

    pub fn take_login_token(&mut self, token: &str) -> Option<UserId> {
        self.login_tokens.remove(token)
    }

    pub fn accept_user(&mut self, user_key: UserKey, user_id: UserId) {
        self.user_key_to_id.insert(user_key, user_id);
    }

    pub fn disconnect_user(
        &mut self,
        commands: &mut Commands,
        naia_server: &mut Server,
        user_key: &UserKey,
    ) -> Option<UserId> {
        let user_id = self.user_key_to_id.remove(user_key)?;
        self.user_data
            .get_mut(&user_id)
            .unwrap()
            .disconnect(commands, naia_server);
        Some(user_id)
    }

    pub fn connect_user(
        &mut self,
        commands: &mut Commands,
        naia_server: &mut Server,
        http_client: &mut HttpClient,
        user_key: &UserKey,
        main_menu_room_key: &RoomKey,
    ) {
        let user_id = self.user_key_to_id(user_key).unwrap();
        if !self.has_user_data(&user_id) {
            self.add_user_data(
                commands,
                naia_server,
                http_client,
                main_menu_room_key,
                &user_id,
            );
        }

        let user_data = self.user_data.get_mut(&user_id).unwrap();
        user_data.connect(commands, naia_server, &user_key);
    }

    pub fn user_key_to_id(&self, user_key: &UserKey) -> Option<UserId> {
        self.user_key_to_id.get(user_key).cloned()
    }

    pub fn make_ready_for_world_connect(&mut self, user_key: &UserKey) -> Result<(), ()> {
        let Some(user_id) = self.user_key_to_id.get(user_key) else {
            return Err(());
        };
        let user_data = self.user_data.get_mut(user_id);
        match user_data {
            Some(user_data) => {
                user_data.make_ready_for_world_connect();
                Ok(())
            }
            None => Err(()),
        }
    }

    // World Connection

    pub fn get_users_ready_to_connect_to_world(
        &mut self,
        world_connect_resend_rate: &Duration,
    ) -> Vec<(UserKey, UserId)> {
        let now = Instant::now();

        let mut worldless_users = Vec::new();
        for (user_key, user_id) in self.user_key_to_id.iter() {
            let user_data = self.user_data.get_mut(user_id).unwrap();
            if user_data.is_world_connected() {
                continue;
            }
            if !user_data.ready_for_world_connect() {
                continue;
            }
            if let Some(last_sent) = user_data.world_connect_last_sent_to_region() {
                let time_since_last_sent = now.duration_since(last_sent);
                if time_since_last_sent >= *world_connect_resend_rate {
                    worldless_users.push((*user_key, *user_id));
                    user_data.set_world_connect_last_sent_to_region(now);
                }
            } else {
                worldless_users.push((*user_key, *user_id));
                user_data.set_world_connect_last_sent_to_region(now);
            }
        }
        worldless_users
    }

    pub fn user_set_world_connected(&mut self, user_key: &UserKey, world_instance_secret: &str) {
        let user_id = self.user_key_to_id(user_key).unwrap();
        let user_data = self.user_data.get_mut(&user_id).unwrap();
        user_data.set_world_connected(world_instance_secret);
    }

    // user entities

    pub(crate) fn has_user_data(&self, user_id: &UserId) -> bool {
        self.user_data.contains_key(user_id)
    }

    pub(crate) fn get_user_entity(&self, user_id: &UserId) -> Option<Entity> {
        self.user_data.get(user_id).map(|data| data.user_entity())
    }

    pub(crate) fn add_user_data(
        &mut self,
        commands: &mut Commands,
        naia_server: &mut Server,
        http_client: &mut HttpClient,
        main_menu_room_key: &RoomKey,
        user_id: &UserId,
    ) {
        if self.user_data.contains_key(user_id) {
            panic!("user data already exists - [userid {:?}]", user_id);
        }

        // convert to entity + component
        let user_entity = commands.spawn_empty().enable_replication(naia_server).id();

        naia_server
            .room_mut(main_menu_room_key)
            .add_entity(&user_entity);

        // send user info req
        let user_info_response_key = self.send_user_info_request(http_client, user_id);
        self.inflight_user_info_requests.insert(*user_id);

        // add user data
        let user_data = UserData::new(user_entity, user_info_response_key);
        self.user_data.insert(*user_id, user_data);
    }

    pub(crate) fn get_or_init_user_entity(
        &mut self,
        commands: &mut Commands,
        naia_server: &mut Server,
        http_client: &mut HttpClient,
        main_menu_room_key: &RoomKey,
        owner_user_id: &UserId
    ) -> Entity {
        if let Some(user_entity) = self.get_user_entity(owner_user_id) {
            user_entity
        } else {
            self.add_user_data(
                commands,
                naia_server,
                http_client,
                main_menu_room_key,
                owner_user_id,
            );

            let user_entity = self.get_user_entity(owner_user_id).unwrap();
            user_entity
        }
    }

    pub(crate) fn user_join_lobby(&mut self, user_id: &UserId, lobby_id: &LobbyId, lobby_member_entity: &Entity) -> (UserKey, Entity) {
        let user_data = self.user_data.get_mut(user_id).unwrap();
        user_data.user_join_lobby(lobby_id, lobby_member_entity)
    }

    // returns (lobby id, lobby member entity)
    pub(crate) fn user_leave_lobby(&mut self, user_id: &UserId) -> (LobbyId, Entity) {
        let user_data = self.user_data.get_mut(user_id).unwrap();
        user_data.user_leave_lobby()
    }

    pub(crate) fn get_user_lobby_id(&self, user_id: &UserId) -> Option<LobbyId> {
        let user_data = self.user_data.get(user_id).unwrap();
        user_data.get_lobby_id()
    }

    pub(crate) fn user_set_online(&mut self, user_id: &UserId, users_q: &mut Query<&mut User>) {
        self.user_set_online_status(user_id, users_q, true);
    }

    pub(crate) fn user_set_offline(&mut self, user_id: &UserId, users_q: &mut Query<&mut User>) {
        self.user_set_online_status(user_id, users_q, false);
    }

    fn user_set_online_status(
        &mut self,
        user_id: &UserId,
        users_q: &mut Query<&mut User>,
        online: bool,
    ) {
        let user_data = self.user_data.get(user_id).unwrap();
        if user_data.user_info_response_key().is_some() {
            // user info req is in flight
            let user_data = self.user_data.get_mut(user_id).unwrap();
            if online {
                user_data.set_online();
            } else {
                user_data.set_offline();
            }
        } else {
            // user info req has been received
            let user_entity = user_data.user_entity();
            let mut user_info = users_q.get_mut(user_entity).unwrap();
            *user_info.online = online;
        }
    }

    pub fn process_in_flight_requests(
        &mut self,
        commands: &mut Commands,
        http_client: &mut HttpClient,
    ) {
        if self.inflight_user_info_requests.is_empty() {
            // no in-flight requests
            return;
        }

        let mut received_responses = Vec::new();
        for nameless_user_id in self.inflight_user_info_requests.iter() {
            let nameless_user_id = *nameless_user_id;
            let user_data = self.user_data.get_mut(&nameless_user_id).unwrap();
            let response_key = user_data.user_info_response_key().unwrap();
            if let Some(response_result) = http_client.recv(&response_key) {
                let host = "session";
                let remote = "auth";
                bevy_http_client::log_util::recv_res(host, remote, UserGetResponse::name());

                match response_result {
                    Ok(response) => {
                        info!("received user get response from auth server");
                        received_responses.push((nameless_user_id, response));
                    }
                    Err(e) => {
                        warn!(
                            "error receiving user get response from social server: {:?}",
                            e.to_string()
                        );
                    }
                }
            }
        }

        for (user_id, received_response) in received_responses {
            self.inflight_user_info_requests.remove(&user_id);

            let user_data = self.user_data.get_mut(&user_id).unwrap();
            let user_is_online = {
                if let Some(online) = user_data.receive_info_response() {
                    online
                } else {
                    false
                }
            };

            let user_entity = user_data.user_entity();
            let user_name = received_response.name;
            commands
                .entity(user_entity)
                .insert(User::new(&user_name, user_is_online));
        }
    }

    pub fn send_user_info_request(
        &mut self,
        http_client: &mut HttpClient,
        user_id: &UserId,
    ) -> ResponseKey<UserGetResponse> {
        let request = UserGetRequest::new(*user_id);

        let host = "session";
        let remote = "auth";
        bevy_http_client::log_util::send_req(host, remote, UserGetRequest::name());
        let response_key = http_client.send(AUTH_SERVER_RECV_ADDR, AUTH_SERVER_PORT, request);

        response_key
    }
}
