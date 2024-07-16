use std::time::Instant;

use bevy_ecs::{system::Commands, entity::Entity};
use naia_bevy_server::{CommandsExt, RoomKey, Server, UserKey};

use bevy_http_client::ResponseKey;

use auth_server_http_proto::UserGetResponse;
use session_server_naia_proto::components::{Selfhood, SelfhoodUser};

pub(crate) struct PrivateUserInfo {
    user_key: Option<UserKey>,
    user_entity: Entity,
    selfhood_entity: Option<Entity>,
    room_key: Option<RoomKey>,

    world_connect_last_sent_to_region: Option<Instant>,
    ready_for_world_connect: bool,

    // LATER this may be used to send meaningful data about a user back to the given world server instance..
    world_instance_secret: Option<String>,

    user_info_response_key: Option<ResponseKey<UserGetResponse>>,
    make_online_after_info: Option<bool>,
}

impl PrivateUserInfo {
    pub fn new(
        user_entity: Entity,
        user_info_response_key: ResponseKey<UserGetResponse>,
    ) -> Self {
        Self {
            user_key: None,
            user_entity,
            selfhood_entity: None,
            room_key: None,

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
        self.user_entity
    }

    pub fn connect(
        &mut self,
        commands: &mut Commands,
        naia_server: &mut Server,
        user_key: &UserKey
    ) {
        if self.user_key.is_some() {
            panic!("User key already set");
        }
        self.user_key = Some(*user_key);

        // make user's private room key
        let user_room_key = naia_server.make_room().key();
        self.room_key = Some(user_room_key);

        // make selfhood entity
        let selfhood_entity = commands
            .spawn_empty()
            .enable_replication(naia_server)
            .id();
        self.selfhood_entity = Some(selfhood_entity);

        naia_server
            .room_mut(&user_room_key)
            // add user to own room
            .add_user(user_key)
            // add selfhood entity to user's room
            .add_entity(&selfhood_entity);

        // add selfhood components
        let mut selfhood_user = SelfhoodUser::new();
        selfhood_user.user_entity.set(naia_server, &self.user_entity);
        commands.entity(selfhood_entity)
            .insert(Selfhood::new())
            .insert(selfhood_user);
    }

    pub fn disconnect(&mut self, commands: &mut Commands, naia_server: &mut Server) {
        if self.user_key.is_none() {
            panic!("User key not set");
        }

        self.user_key = None;

        // destroy user's room
        let user_room_key = self.room_key.take().unwrap();
        naia_server.room_mut(&user_room_key).destroy();

        // despawn selfhood
        let selfhood_entity = self.selfhood_entity.take().unwrap();
        commands.entity(selfhood_entity).despawn();
    }
}
