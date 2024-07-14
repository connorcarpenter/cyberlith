use std::time::Instant;

use bevy_ecs::entity::Entity;

use naia_bevy_server::UserKey;

use bevy_http_client::ResponseKey;

use auth_server_http_proto::UserGetResponse;

pub(crate) struct PrivateUserInfo {
    user_key: Option<UserKey>,
    public_entity: Entity,

    world_connect_last_sent_to_region: Option<Instant>,
    ready_for_world_connect: bool,

    // LATER this may be used to send meaningful data about a user back to the given world server instance..
    world_instance_secret: Option<String>,

    user_info_response_key: Option<ResponseKey<UserGetResponse>>,
    make_online_after_info: Option<bool>,
}

impl PrivateUserInfo {
    pub fn new(
        public_entity: Entity,
        user_info_response_key: ResponseKey<UserGetResponse>,
    ) -> Self {
        Self {
            user_key: None,
            public_entity,

            world_connect_last_sent_to_region: None,
            ready_for_world_connect: false,

            world_instance_secret: None, // tells us whether user is connected

            user_info_response_key: Some(user_info_response_key),
            make_online_after_info: None,
        }
    }

    pub(crate) fn set_online(&mut self) {
        self.make_online_after_info = Some(true);
    }

    pub(crate) fn set_offline(&mut self) {
        self.make_online_after_info = Some(false);
    }

    pub fn user_info_response_key(&self) -> Option<ResponseKey<UserGetResponse>> {
        self.user_info_response_key
    }

    pub(crate) fn receive_info_response(&mut self) -> Option<bool> {
        self.user_info_response_key = None;

        let output = self.make_online_after_info;
        self.make_online_after_info = None;
        output
    }

    pub fn ready_for_world_connect(&self) -> bool {
        self.ready_for_world_connect
    }

    pub fn make_ready_for_world_connect(&mut self) {
        self.ready_for_world_connect = true;
    }

    pub fn world_connect_last_sent_to_region(&self) -> Option<Instant> {
        self.world_connect_last_sent_to_region
    }

    pub fn set_world_connect_last_sent_to_region(&mut self, instant: Instant) {
        self.world_connect_last_sent_to_region = Some(instant);
    }

    pub fn is_world_connected(&self) -> bool {
        self.world_instance_secret.is_some()
    }

    pub fn set_world_connected(&mut self, world_instance_secret: &str) {
        self.world_instance_secret = Some(world_instance_secret.to_string());
    }

    pub fn user_entity(&self) -> Entity {
        self.public_entity
    }

    pub fn add_user_key(&mut self, user_key: &UserKey) {
        self.user_key = Some(*user_key);
    }

    pub fn remove_user_key(&mut self) {
        self.user_key = None;
    }
}
