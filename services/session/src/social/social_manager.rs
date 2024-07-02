use std::collections::VecDeque;

use bevy_ecs::{
    change_detection::ResMut,
    entity::Entity,
    system::{Commands, Res, Resource},
};

use naia_bevy_server::{CommandsExt, RoomKey, Server, UserKey};

use bevy_http_client::{ApiRequest, ApiResponse, HttpClient, ResponseKey};
use logging::{info, warn};

use auth_server_types::UserId;

use session_server_http_proto::SocialUserPatch;
use session_server_naia_proto::components::GlobalChatMessage;

use social_server_http_proto::{GlobalChatSendMessageRequest, GlobalChatSendMessageResponse, UserDisconnectedRequest, UserDisconnectedResponse};
use social_server_types::{GlobalChatMessageId, Timestamp};

use crate::{session_instance::SessionInstance, user::UserManager};

enum QueuedRequest {
    // user_id
    UserSendDisconnect(UserId),
    // user id, message
    GlobalChatSendMessage(UserKey, String),
}

enum InFlightRequest {
    // user id
    UserSendDisconnect(UserId, ResponseKey<UserDisconnectedResponse>),
    // sending user id, message, response key
    GlobalChatSendMessage(UserId, String, ResponseKey<GlobalChatSendMessageResponse>),
}

#[derive(Resource)]
pub struct SocialManager {
    social_server_opt: Option<(String, u16)>,
    queued_requests: Vec<QueuedRequest>,
    in_flight_requests: Vec<InFlightRequest>,

    global_chat_room_key: Option<RoomKey>,
    global_chat_log: VecDeque<Entity>,
}

impl SocialManager {
    pub fn new() -> Self {
        Self {
            social_server_opt: None,
            in_flight_requests: Vec::new(),
            queued_requests: Vec::new(),

            global_chat_room_key: None,
            global_chat_log: VecDeque::new(),
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
    pub fn startup(mut naia_server: Server, mut social_manager: ResMut<Self>) {
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
        mut user_manager: ResMut<UserManager>,
    ) {
        social_manager.process_in_flight_requests(
            &mut commands,
            &mut naia_server,
            &mut http_client,
            &mut user_manager,
        );
        social_manager.process_queued_requests(&mut http_client, &session_instance, &user_manager);
    }

    //

    pub fn get_global_chat_room_key(&self) -> RoomKey {
        self.global_chat_room_key.unwrap()
    }

    pub fn process_queued_requests(
        &mut self,
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
                        http_client,
                        user_manager,
                        session_instance,
                        &user_key,
                        &message,
                    );
                }
                QueuedRequest::UserSendDisconnect(user_id) => {
                    self.send_user_disconnect_req(http_client, session_instance, &user_id);
                }
            }
        }
    }

    pub fn process_in_flight_requests(
        &mut self,
        commands: &mut Commands,
        naia_server: &mut Server,
        http_client: &mut HttpClient,
        user_manager: &mut UserManager,
    ) {
        if self.in_flight_requests.is_empty() {
            // no in-flight requests
            return;
        }

        let mut continuing_requests = Vec::new();
        let in_flight_requests = std::mem::take(&mut self.in_flight_requests);

        for req in in_flight_requests {
            match &req {
                InFlightRequest::GlobalChatSendMessage(sending_user_id, message, response_key) => {
                    if let Some(response_result) = http_client.recv(&response_key) {
                        let host = "session";
                        let remote = "social";
                        bevy_http_client::log_util::recv_res(
                            host,
                            remote,
                            GlobalChatSendMessageResponse::name(),
                        );

                        match response_result {
                            Ok(response) => {
                                // info!("received global chat send message response from social server");
                                let global_chat_id = response.global_chat_message_id;
                                let timestamp = response.timestamp;

                                // log the message
                                self.log_global_chat_message(
                                    commands,
                                    naia_server,
                                    user_manager,
                                    &global_chat_id,
                                    &timestamp,
                                    sending_user_id,
                                    message,
                                );
                            }
                            Err(e) => {
                                warn!("error receiving global chat send message response from social server: {:?}", e.to_string());
                            }
                        }
                    } else {
                        continuing_requests.push(req);
                    }
                }
                InFlightRequest::UserSendDisconnect(user_id, response_key) => {
                    if let Some(response_result) = http_client.recv(&response_key) {
                        let host = "session";
                        let remote = "social";
                        bevy_http_client::log_util::recv_res(
                            host,
                            remote,
                            UserDisconnectedResponse::name(),
                        );

                        match response_result {
                            Ok(_response) => {
                                info!("received user disconnect response from social server");

                                user_manager.user_set_offline(
                                    user_id,
                                );
                            }
                            Err(e) => {
                                warn!("error receiving user disconnect response from social server: {:?}", e.to_string());
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

    // Users

    pub(crate) fn patch_users(
        &mut self,
        commands: &mut Commands,
        naia_server: &mut Server,
        user_manager: &mut UserManager,
        user_patches: &Vec<SocialUserPatch>,
    ) {
        for user_patch in user_patches {
            match user_patch {
                SocialUserPatch::Add(user_id, user_name) => {
                    info!("adding user - [userid {:?}]:(`{:?}`)", user_id, user_name);

                    if user_manager.has_user_data(user_id) {
                        // warn!("user already exists - [userid {:?}]", user_id);
                        continue;
                    }

                    user_manager.add_user_data(
                        commands,
                        naia_server,
                        &self.get_global_chat_room_key(),
                        user_id,
                        user_name
                    );
                }
                SocialUserPatch::Remove(user_id) => {
                    info!("removing user - [userid {:?}]", user_id);

                    user_manager.user_set_offline(user_id);
                }
            }
        }
    }

    pub fn send_user_disconnect_req(
        &mut self,
        http_client: &mut HttpClient,
        session_instance: &SessionInstance,
        user_id: &UserId,
    ) {
        let Some((social_server_addr, social_server_port)) = self.get_social_server_url() else {
            warn!("user disconnected but no social server is available!");

            let qr = QueuedRequest::UserSendDisconnect(*user_id);
            self.queued_requests.push(qr);

            return;
        };

        // info!("sending user disconnect request to social server - [userid {:?}]", user_id);
        let request = UserDisconnectedRequest::new(
            session_instance.instance_secret(),
            *user_id,
        );

        let host = "session";
        let remote = "social";
        bevy_http_client::log_util::send_req(host, remote, UserDisconnectedRequest::name());
        let response_key = http_client.send(&social_server_addr, social_server_port, request);

        let ifr = InFlightRequest::UserSendDisconnect(
            *user_id,
            response_key,
        );
        self.in_flight_requests.push(ifr);
    }

    // Global Chat

    pub fn send_global_chat_message(
        &mut self,
        http_client: &mut HttpClient,
        user_manager: &UserManager,
        session_instance: &SessionInstance,
        sending_user_key: &UserKey,
        message: &str,
    ) {
        let Some(sending_user_id) = user_manager.user_key_to_id(sending_user_key) else {
            warn!("User not found: {:?}", sending_user_key);
            return;
        };

        let Some((social_server_addr, social_server_port)) = self.get_social_server_url() else {
            warn!("received global chat message but no social server is available!");

            let qr = QueuedRequest::GlobalChatSendMessage(*sending_user_key, message.to_string());
            self.queued_requests.push(qr);

            return;
        };

        // info!("sending global chat send message request to social server - [userid {:?}]:(`{:?}`)", sending_user_id, message);
        let request = GlobalChatSendMessageRequest::new(
            session_instance.instance_secret(),
            sending_user_id,
            message,
        );

        let host = "session";
        let remote = "social";
        bevy_http_client::log_util::send_req(host, remote, GlobalChatSendMessageRequest::name());
        let response_key = http_client.send(&social_server_addr, social_server_port, request);

        let ifr = InFlightRequest::GlobalChatSendMessage(
            sending_user_id,
            message.to_string(),
            response_key,
        );
        self.in_flight_requests.push(ifr);
    }

    fn log_global_chat_message(
        &mut self,
        commands: &mut Commands,
        naia_server: &mut Server,
        user_manager: &UserManager,
        global_chat_id: &GlobalChatMessageId,
        timestamp: &Timestamp,
        sending_user_id: &UserId,
        message: &str,
    ) {
        let Some(user_entity) = user_manager.get_user_entity(sending_user_id) else {
            panic!("User not found: {:?}", sending_user_id);
        };
        let global_chat_message = GlobalChatMessage::new(
            naia_server,
            *global_chat_id,
            *timestamp,
            user_entity,
            message,
        );
        let global_chat_message_id = commands
            .spawn_empty()
            .enable_replication(naia_server)
            .insert(global_chat_message)
            .id();

        naia_server
            .room_mut(&self.get_global_chat_room_key())
            .add_entity(&global_chat_message_id);

        // add to local log
        self.global_chat_log.push_back(global_chat_message_id);

        // remove oldest messages if we have too many
        if self.global_chat_log.len() > 100 {
            let entity_to_delete = self.global_chat_log.pop_front().unwrap();
            commands.entity(entity_to_delete).despawn();
        }
    }

    pub(crate) fn patch_global_chat_messages(
        &mut self,
        commands: &mut Commands,
        naia_server: &mut Server,
        user_manager: &UserManager,
        new_messages: &Vec<(GlobalChatMessageId, Timestamp, UserId, String)>,
    ) {
        for (msg_id, timestamp, user_id, message) in new_messages {

            // log the message
            self.log_global_chat_message(
                commands,
                naia_server,
                user_manager,
                msg_id,
                timestamp,
                user_id,
                message,
            );
        }
    }
}
