use std::collections::VecDeque;

use bevy_ecs::{entity::Entity, system::{Commands, Resource, Res}, change_detection::ResMut};

use naia_bevy_server::{CommandsExt, RoomKey, Server, UserKey};
use auth_server_types::UserId;

use bevy_http_client::{HttpClient, ResponseKey};
use logging::{info, warn};

use session_server_naia_proto::components::GlobalChatMessage;
use social_server_http_proto::{GlobalChatSendMessageRequest, GlobalChatSendMessageResponse};

use crate::{user::UserManager, session_instance::SessionInstance};

enum QueuedRequest {
    // user id, message
    GlobalChatSendMessage(UserKey, String),
}

enum InFlightRequest {
    GlobalChatSendMessage(ResponseKey<GlobalChatSendMessageResponse>),
}

#[derive(Resource)]
pub struct SocialManager {
    social_server_opt: Option<(String, u16)>,
    queued_requests: Vec<QueuedRequest>,
    in_flight_requests: Vec<InFlightRequest>,

    global_chat_room_key: Option<RoomKey>,
    global_chat_log: VecDeque<Entity>,
    next_timestamp: u16,
}

impl SocialManager {
    pub fn new() -> Self {
        Self {
            social_server_opt: None,
            in_flight_requests: Vec::new(),
            queued_requests: Vec::new(),

            global_chat_room_key: None,
            global_chat_log: VecDeque::new(),
            next_timestamp: 0,
        }
    }

    // Social Server

    pub fn set_social_server(&mut self, addr: &str, port: u16) {
        self.social_server_opt = Some((addr.to_string(), port));
    }

    pub fn clear_social_server(&mut self) {
        self.social_server_opt = None;
    }

    pub fn get_social_server_url(&self) -> Option<(String, u16)> {
        self.social_server_opt
            .as_ref()
            .map(|(addr, port)| (addr.clone(), *port))
    }

    // used as a system
    pub fn startup(
        mut naia_server: Server,
        mut social_manager: ResMut<Self>,
    ) {
        let global_chat_room_key = naia_server.make_room().key();
        social_manager.global_chat_room_key = Some(global_chat_room_key);
    }

    // used as a system
    pub fn update(
        mut commands: Commands,
        mut naia_server: Server,
        mut http_client: ResMut<HttpClient>,
        mut social_manager: ResMut<Self>,
        session_instance: Res<SessionInstance>,
        user_manager: Res<UserManager>,
    ) {
        social_manager.process_in_flight_requests(
            &mut http_client,
        );
        social_manager.process_queued_requests(
            &mut commands, &mut naia_server, &mut http_client, &session_instance, &user_manager,
        );
    }

    //

    pub fn get_global_chat_room_key(&self) -> RoomKey {
        self.global_chat_room_key.unwrap()
    }

    pub fn process_queued_requests(
        &mut self,
        commands: &mut Commands,
        naia_server: &mut Server,
        http_client: &mut HttpClient,
        session_instance: &SessionInstance,
        user_manager: &UserManager,
    ) {
        if self.queued_requests.is_empty() {
            // no queued assets
            return;
        }
        if self.get_social_server_url().is_none() {
            // it's okay to wait until the social server is available
            return;
        };

        let queued_requests = std::mem::take(&mut self.queued_requests);
        for request in queued_requests {
            match request {
                QueuedRequest::GlobalChatSendMessage(user_key, message) => {
                    self.send_global_chat_message(
                        commands,
                        naia_server,
                        http_client,
                        user_manager,
                        session_instance,
                        &user_key,
                        &message
                    );
                }
            }
        }
    }

    pub fn process_in_flight_requests(
        &mut self,
        http_client: &mut HttpClient,
    ) {
        if self.in_flight_requests.is_empty() {
            // no in-flight requests
            return;
        }

        let mut continuing_requests = Vec::new();
        let in_flight_requests = std::mem::take(&mut self.in_flight_requests);

        for req in in_flight_requests {
            match req {
                InFlightRequest::GlobalChatSendMessage(response_key) => {
                    if let Some(response_result) = http_client.recv(&response_key) {
                        match response_result {
                            Ok(_response) => {
                                info!("received global chat send message response from social server");
                            }
                            Err(e) => {
                                warn!("error receiving global chat send message response from social server: {:?}", e.to_string());
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

    // Global Chat

    pub fn send_global_chat_message(
        &mut self,
        commands: &mut Commands,
        naia_server: &mut Server,
        http_client: &mut HttpClient,
        user_manager: &UserManager,
        session_instance: &SessionInstance,
        sending_user_key: &UserKey,
        message: &str,
    ) {
        let Some(user_data) = user_manager.get_user_data(sending_user_key) else {
            warn!("User not found: {:?}", sending_user_key);
            return;
        };
        let sending_user_id = user_data.user_id;

        let Some((social_server_addr, social_server_port)) = self.get_social_server_url() else {
            warn!("received global chat message but no social server is available!");

            let qr = QueuedRequest::GlobalChatSendMessage(*sending_user_key, message.to_string());
            self.queued_requests.push(qr);

            return;
        };

        info!("sending global chat send message request to social server - [userid {:?}]:(`{:?}`)", sending_user_id, message);
        let request = GlobalChatSendMessageRequest::new(session_instance.instance_secret(), sending_user_id, message);
        let response_key = http_client.send(&social_server_addr, social_server_port, request);

        let ifr = InFlightRequest::GlobalChatSendMessage(response_key);
        self.in_flight_requests.push(ifr);

        // log the message
        self.log_global_chat_message(commands, naia_server, sending_user_id, message);
    }

    fn log_global_chat_message(
        &mut self,
        commands: &mut Commands,
        naia_server: &mut Server,
        sending_user_id: UserId,
        message: &str,
    ) {
        // convert to entity + component
        let timestamp = self.get_next_timestamp();
        let global_chat_message_id = commands
            .spawn_empty()
            .enable_replication(naia_server)
            .insert(GlobalChatMessage::new(timestamp, sending_user_id, message))
            .id();

        naia_server.room_mut(&self.get_global_chat_room_key()).add_entity(&global_chat_message_id);

        // add to local log
        self.global_chat_log.push_back(global_chat_message_id);

        // remove oldest messages if we have too many
        if self.global_chat_log.len() > 100 {
            let entity_to_delete = self.global_chat_log.pop_front().unwrap();
            commands.entity(entity_to_delete).despawn();
        }
    }
    fn get_next_timestamp(&mut self) -> u16 {
        let next_timestamp = self.next_timestamp;
        self.next_timestamp = self.next_timestamp.wrapping_add(1);
        next_timestamp
    }

    pub(crate) fn patch_global_chat_messages(
        &mut self,
        commands: &mut Commands,
        naia_server: &mut Server,
        new_messages: &Vec<(UserId, String)>
    ) {
        for (user_id, message) in new_messages {
            self.log_global_chat_message(commands, naia_server, *user_id, message);
        }
    }
}
