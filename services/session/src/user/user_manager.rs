use std::collections::{HashMap};

use bevy_ecs::{
    change_detection::ResMut,
    entity::Entity,
    prelude::{Commands, Query},
    system::Resource,
};

use naia_bevy_server::{CommandsExt, RoomKey, Server, UserKey};

use bevy_http_client::{HttpClient};

use auth_server_types::UserId;
use session_server_naia_proto::components::User;
use social_server_types::LobbyId;

use crate::user::{user_data::UserData, user_info_service::UserInfoService, user_login_token_store::UserLoginTokenStore};

#[derive(Resource)]
pub struct UserManager {
    login_token_store: UserLoginTokenStore,
    user_key_to_id: HashMap<UserKey, UserId>,
    user_data: HashMap<UserId, UserData>,
    user_info_service: UserInfoService,
}

impl UserManager {
    pub fn new() -> Self {
        Self {
            login_token_store: UserLoginTokenStore::new(),
            user_key_to_id: HashMap::new(),
            user_data: HashMap::new(),
            user_info_service: UserInfoService::new(),
        }
    }

    // used as a system
    pub fn update(
        mut commands: Commands,
        mut http_client: ResMut<HttpClient>,
        mut user_manager: ResMut<Self>,
    ) {
        if let Some(responses) = user_manager.user_info_service.process_in_flight_requests(&mut http_client) {
            for (user_id, response) in responses {
                let user_data = user_manager.user_data.get_mut(&user_id).unwrap();
                let user_is_online = {
                    if let Some(online) = user_data.receive_info_response() {
                        online
                    } else {
                        false
                    }
                };

                let user_entity = user_data.user_entity();
                let user_name = response.name;
                commands
                    .entity(user_entity)
                    .insert(User::new(&user_name, user_is_online));
            }
        }
    }

    // Client login

    pub fn recv_login_token(&mut self, user_id: &UserId, token: &str) {
        self.login_token_store.recv_login_token(user_id, token);
    }

    pub fn spend_login_token(&mut self, token: &str) -> Option<UserId> {
        self.login_token_store.spend_login_token(token)
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

    pub fn user_id_to_key(&self, user_id: &UserId) -> Option<UserKey> {
        self.user_data.get(user_id)?.user_key()
    }

    // World Connection

    pub fn user_has_world_connection(&self, user_key: &UserKey) -> bool {
        let user_id = self.user_key_to_id(user_key).unwrap();
        let user_data = self.user_data.get(&user_id).unwrap();
        user_data.get_world_connected()
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
        self.user_info_service.send_user_info_request(http_client, user_id);

        // add user data
        let user_data = UserData::new(user_entity);
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
        if user_data.requesting_info() {
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
}
