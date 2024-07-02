use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use bevy_ecs::{system::Resource, prelude::Commands, entity::Entity};

use naia_bevy_server::{CommandsExt, RoomKey, Server, UserKey};

use auth_server_types::UserId;
use session_server_naia_proto::components::PublicUserInfo;

use crate::user::user_data::UserData;

#[derive(Resource)]
pub struct UserManager {
    login_tokens: HashMap<String, (UserId, String)>,
    user_key_to_id: HashMap<UserKey, UserId>,
    user_data: HashMap<UserId, UserData>,
}

impl UserManager {
    pub fn new() -> Self {
        Self {
            login_tokens: HashMap::new(),
            user_key_to_id: HashMap::new(),
            user_data: HashMap::new(),
        }
    }

    // Client login

    pub fn add_login_token(&mut self, user_id: &UserId, user_name: &str, token: &str) {
        self.login_tokens.insert(token.to_string(), (user_id.clone(), user_name.to_string()));
    }

    pub fn take_login_token(&mut self, token: &str) -> Option<(UserId, String)> {
        self.login_tokens.remove(token)
    }

    pub fn add_connected_user(
        &mut self,
        commands: &mut Commands,
        naia_server: &mut Server,
        global_chat_room_key: &RoomKey,
        user_key: UserKey,
        user_id: UserId,
        user_name: String
    ) {
        self.user_key_to_id.insert(user_key, user_id);

        if self.has_user_data(&user_id) {
            // warn!("user already exists - [userid {:?}]", user_id);
            return;
        }

        self.add_user_data(
            commands,
            naia_server,
            global_chat_room_key,
            &user_id,
            &user_name
        );

        let user_data = self.user_data.get_mut(&user_id).unwrap();
        user_data.add_user_key(&user_key);
    }

    pub fn remove_connected_user(&mut self, user_key: &UserKey) -> Option<UserId> {
        let user_id = self.user_key_to_id.remove(user_key)?;
        self.user_data.get_mut(&user_id).unwrap().remove_user_key();
        Some(user_id)
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

    fn add_user_public_entity(
        &mut self,
        commands: &mut Commands,
        naia_server: &mut Server,
        global_chat_room_key: &RoomKey,
        user_name: &str
    ) -> Entity {
        // info!("adding present user - [userid {:?}]:(`{:?}`)", user_id, user_name);
        // convert to entity + component
        let user_entity = commands
            .spawn_empty()
            .enable_replication(naia_server)
            .insert(PublicUserInfo::new(
                user_name,
            ))
            .id();

        naia_server
            .room_mut(global_chat_room_key)
            .add_entity(&user_entity);

        user_entity
    }

    pub(crate) fn add_user_data(
        &mut self,
        commands: &mut Commands,
        naia_server: &mut Server,
        global_chat_room_key: &RoomKey,
        user_id: &UserId,
        user_name: &str,
    ) {
        let user_public_entity = self.add_user_public_entity(
            commands,
            naia_server,
            global_chat_room_key,
            user_name,
        );

        let user_data = UserData::new(user_public_entity);
        self.user_data.insert(*user_id, user_data);
    }

    pub(crate) fn user_set_offline(
        &mut self,
        user_id: &UserId,
    ) {
        let user_data = self.user_data.get_mut(user_id).unwrap();
        user_data.set_offline();
    }
}