use std::{
    collections::HashMap,
    time::Duration,
};

use bevy_ecs::system::Resource;

use auth_server_types::UserId;
use naia_bevy_server::{RoomKey, UserKey};

use crate::region_connection::RegionServerState;

pub struct UserData {
    pub user_id: UserId,
    pub session_server_addr: String,
    pub session_server_port: u16,
}

impl UserData {
    fn new(user_id: UserId, session_server_addr: &str, session_server_port: u16) -> Self {
        Self {
            user_id,
            session_server_addr: session_server_addr.to_string(),
            session_server_port,
        }
    }
}

#[derive(Resource)]
pub struct Global {
    instance_secret: String,
    pub region_server: RegionServerState,
    login_tokens: HashMap<String, UserData>,
    users: HashMap<UserKey, UserData>,
    main_room_key: RoomKey,
}

impl Global {
    pub fn new(
        instance_secret: &str,
        main_room_key: RoomKey,
        registration_resend_rate: Duration,
        region_server_disconnect_timeout: Duration,
    ) -> Self {
        Self {
            instance_secret: instance_secret.to_string(),
            region_server: RegionServerState::new(
                registration_resend_rate,
                region_server_disconnect_timeout,
            ),
            login_tokens: HashMap::new(),
            users: HashMap::new(),
            main_room_key,
        }
    }

    pub fn instance_secret(&self) -> &str {
        &self.instance_secret
    }

    pub fn disconnect_region_server(&mut self) {
        self.region_server.set_disconnected();

        // clear login tokens
        self.login_tokens.clear();

        // clear users
        self.users.clear();
    }

    // Client login

    pub fn add_login_token(
        &mut self,
        session_server_addr: &str,
        session_server_port: u16,
        user_id: UserId,
        token: &str,
    ) {
        self.login_tokens.insert(
            token.to_string(),
            UserData::new(user_id, session_server_addr, session_server_port),
        );
    }

    pub fn take_login_token(&mut self, token: &str) -> Option<UserData> {
        self.login_tokens.remove(token)
    }

    pub fn add_user(&mut self, user_key: &UserKey, user_data: UserData) {
        self.users.insert(user_key.clone(), user_data);
    }

    pub fn get_user_id(&self, user_key: &UserKey) -> Option<UserId> {
        let user_data = self.users.get(user_key)?;
        Some(user_data.user_id)
    }

    pub fn get_user_session_server(&self, user_key: &UserKey) -> Option<(String, u16)> {
        let user_data = self.users.get(user_key)?;
        Some((
            user_data.session_server_addr.clone(),
            user_data.session_server_port,
        ))
    }
    //

    pub fn main_room_key(&self) -> RoomKey {
        self.main_room_key
    }
}
