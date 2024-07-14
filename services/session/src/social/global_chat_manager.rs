use std::collections::VecDeque;

use bevy_ecs::{entity::Entity, system::Commands};

use naia_bevy_server::{CommandsExt, RoomKey, Server, UserKey};

use bevy_http_client::{ApiRequest, ApiResponse, HttpClient, ResponseKey};
use logging::warn;

use auth_server_types::UserId;

use session_server_naia_proto::components::MessagePublic;

use social_server_http_proto::{GlobalChatSendMessageRequest, GlobalChatSendMessageResponse};
use social_server_types::{MessageId, Timestamp};

use crate::{session_instance::SessionInstance, user::UserManager};

struct GlobalChatReqQueued(UserKey, String);
struct GlobalChatReqInFlight(UserId, String, ResponseKey<GlobalChatSendMessageResponse>);

pub(crate) struct GlobalChatManager {
    queued_requests: Vec<GlobalChatReqQueued>,
    in_flight_requests: Vec<GlobalChatReqInFlight>,

    global_chat_room_key: Option<RoomKey>,
    global_chat_log: VecDeque<Entity>,
}

impl GlobalChatManager {
    pub(crate) fn new() -> Self {
        Self {
            queued_requests: Vec::new(),
            in_flight_requests: Vec::new(),

            global_chat_room_key: None,
            global_chat_log: VecDeque::new(),
        }
    }

    pub(crate) fn startup(&mut self, naia_server: &mut Server) {
        let global_chat_room_key = naia_server.make_room().key();
        self.global_chat_room_key = Some(global_chat_room_key);
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

    pub(crate) fn room_key(&self) -> RoomKey {
        self.global_chat_room_key.unwrap()
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
            self.send_global_chat_message(
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
        user_presence_room_key: &RoomKey,
    ) {
        if self.in_flight_requests.is_empty() {
            // no in-flight requests
            return;
        }

        let mut continuing_requests = Vec::new();
        let in_flight_requests = std::mem::take(&mut self.in_flight_requests);

        for req in in_flight_requests {
            let GlobalChatReqInFlight(sending_user_id, message, response_key) = &req;

            if let Some(response_result) = http_client.recv(response_key) {
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
                            http_client,
                            user_manager,
                            user_presence_room_key,
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

        self.in_flight_requests = continuing_requests;
    }

    pub(crate) fn send_global_chat_message(
        &mut self,
        http_client: &mut HttpClient,
        user_manager: &UserManager,
        social_server_url: Option<&(String, u16)>,
        session_instance: &SessionInstance,
        sending_user_key: &UserKey,
        message: &str,
    ) {
        let Some(sending_user_id) = user_manager.user_key_to_id(sending_user_key) else {
            warn!("User not found: {:?}", sending_user_key);
            return;
        };

        let Some((social_server_addr, social_server_port)) = social_server_url else {
            warn!("received global chat message but no social server is available!");

            self.queued_requests
                .push(GlobalChatReqQueued(*sending_user_key, message.to_string()));

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
        let response_key = http_client.send(social_server_addr, *social_server_port, request);

        self.in_flight_requests.push(GlobalChatReqInFlight(
            sending_user_id,
            message.to_string(),
            response_key,
        ));

        return;
    }

    fn log_global_chat_message(
        &mut self,
        commands: &mut Commands,
        naia_server: &mut Server,
        http_client: &mut HttpClient,
        user_manager: &mut UserManager,
        user_presence_room_key: &RoomKey,
        global_chat_id: &MessageId,
        timestamp: &Timestamp,
        sending_user_id: &UserId,
        message: &str,
    ) {
        // spawn message entity
        let global_chat_message_entity =
            commands.spawn_empty().enable_replication(naia_server).id();
        let mut global_chat_message = MessagePublic::new(*global_chat_id, *timestamp, message);

        // add to global chat room
        let global_chat_room_key = self.room_key();
        naia_server
            .room_mut(&global_chat_room_key)
            .add_entity(&global_chat_message_entity);

        // add to local log
        self.global_chat_log.push_back(global_chat_message_entity);

        // remove oldest messages if we have too many
        if self.global_chat_log.len() > 100 {
            let entity_to_delete = self.global_chat_log.pop_front().unwrap();
            commands.entity(entity_to_delete).despawn();
        }

        let user_entity = {
            if let Some(user_entity) = user_manager.get_user_entity(sending_user_id) {
                user_entity
            } else {
                user_manager.add_user_data(
                    commands,
                    naia_server,
                    http_client,
                    user_presence_room_key,
                    sending_user_id,
                );

                let user_entity = user_manager.get_user_entity(sending_user_id).unwrap();
                user_entity
            }
        };

        global_chat_message
            .owner_user_entity
            .set(naia_server, &user_entity);
        commands
            .entity(global_chat_message_entity)
            .insert(global_chat_message);
    }

    pub(crate) fn patch_global_chat_messages(
        &mut self,
        commands: &mut Commands,
        naia_server: &mut Server,
        http_client: &mut HttpClient,
        user_manager: &mut UserManager,
        user_presence_room_key: &RoomKey,
        new_messages: &Vec<(MessageId, Timestamp, UserId, String)>,
    ) {
        for (msg_id, timestamp, user_id, message) in new_messages {
            // log the message
            self.log_global_chat_message(
                commands,
                naia_server,
                http_client,
                user_manager,
                user_presence_room_key,
                msg_id,
                timestamp,
                user_id,
                message,
            );
        }
    }
}
