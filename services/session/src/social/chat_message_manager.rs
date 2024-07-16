use std::collections::{HashMap, VecDeque};

use bevy_ecs::{entity::Entity, system::Commands};

use naia_bevy_server::{CommandsExt, RoomKey, Server, UserKey};

use bevy_http_client::{ApiRequest, ApiResponse, HttpClient, ResponseKey};
use logging::warn;
use auth_server_types::UserId;
use session_server_naia_proto::components::{ChatMessage, ChatMessageGlobal, ChatMessageLocal};
use social_server_http_proto::{GlobalChatSendMessageRequest, GlobalChatSendMessageResponse, MatchLobbySendMessageRequest, MatchLobbySendMessageResponse};
use social_server_types::{LobbyId, MessageId, Timestamp};

use crate::{social::lobby_manager::LobbyManager, session_instance::SessionInstance, user::UserManager};

enum ChatMessageReqQueued {
    GlobalChatSendMessage(UserKey, String),
    LobbyChatSendMessage(UserKey, String),
}

enum ChatMessageReqInFlight {
    GlobalChatSendMessage(UserId, String, ResponseKey<GlobalChatSendMessageResponse>),
    LobbyChatSendMessage(UserId, String, ResponseKey<MatchLobbySendMessageResponse>),
}

pub(crate) struct ChatMessageManager {
    queued_requests: Vec<ChatMessageReqQueued>,
    in_flight_requests: Vec<ChatMessageReqInFlight>,

    global_chat_message_entities: VecDeque<Entity>,
    lobby_chat_message_entities: HashMap<LobbyId, VecDeque<Entity>>,
}

impl ChatMessageManager {
    pub(crate) fn new() -> Self {
        Self {
            queued_requests: Vec::new(),
            in_flight_requests: Vec::new(),

            global_chat_message_entities: VecDeque::new(),
            lobby_chat_message_entities: HashMap::new(),
        }
    }

    pub(crate) fn update(
        &mut self,
        commands: &mut Commands,
        naia_server: &mut Server,
        http_client: &mut HttpClient,
        user_manager: &mut UserManager,
        lobby_manager: &LobbyManager,
        social_server_url: &Option<(String, u16)>,
        session_instance: &SessionInstance,
        main_menu_room_key: &RoomKey,
    ) {
        self.process_in_flight_requests(
            commands,
            naia_server,
            http_client,
            user_manager,
            lobby_manager,
            main_menu_room_key,
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
                ChatMessageReqQueued::GlobalChatSendMessage(user_key, message) => {
                    self.send_global_chat_message(
                        http_client,
                        user_manager,
                        social_server_url.as_ref(),
                        session_instance,
                        &user_key,
                        &message,
                    );
                }
                ChatMessageReqQueued::LobbyChatSendMessage(user_key, message) => {
                    self.send_lobby_chat_message(
                        http_client,
                        user_manager,
                        social_server_url.as_ref(),
                        session_instance,
                        &user_key,
                        &message,
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
        lobby_manager: &LobbyManager,
        main_menu_room_key: &RoomKey,
    ) {
        if self.in_flight_requests.is_empty() {
            // no in-flight requests
            return;
        }

        let mut continuing_requests: Vec<ChatMessageReqInFlight> = Vec::new();
        let in_flight_requests = std::mem::take(&mut self.in_flight_requests);

        for req in in_flight_requests {
            match &req {
                ChatMessageReqInFlight::GlobalChatSendMessage(
                    sending_user_id,
                    message,
                    response_key,
                ) => {
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
                                let global_chat_id = response.message_id;
                                let timestamp = response.timestamp;

                                // log the message
                                self.log_global_chat_message(
                                    commands,
                                    naia_server,
                                    http_client,
                                    user_manager,
                                    main_menu_room_key,
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
                ChatMessageReqInFlight::LobbyChatSendMessage(
                    sending_user_id,
                    message,
                    response_key,
                ) => {
                    if let Some(response_result) = http_client.recv(&response_key) {
                        let host = "session";
                        let remote = "social";
                        bevy_http_client::log_util::recv_res(
                            host,
                            remote,
                            MatchLobbySendMessageResponse::name(),
                        );

                        match response_result {
                            Ok(response) => {
                                // info!("received lobby chat send message response from social server");
                                let message_id = response.message_id;
                                let timestamp = response.timestamp;

                                // log the message
                                self.log_lobby_chat_message(
                                    commands,
                                    naia_server,
                                    http_client,
                                    user_manager,
                                    lobby_manager,
                                    main_menu_room_key,
                                    &message_id,
                                    &timestamp,
                                    sending_user_id,
                                    message,
                                );
                            }
                            Err(e) => {
                                warn!("error receiving lobby chat send message response from social server: {:?}", e.to_string());
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
                .push(ChatMessageReqQueued::GlobalChatSendMessage(*sending_user_key, message.to_string()));

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

        self.in_flight_requests.push(ChatMessageReqInFlight::GlobalChatSendMessage(
            sending_user_id,
            message.to_string(),
            response_key,
        ));

        return;
    }

    pub(crate) fn send_lobby_chat_message(
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
                .push(ChatMessageReqQueued::LobbyChatSendMessage(*sending_user_key, message.to_string()));

            return;
        };

        // info!("sending lobby chat send message request to social server - [userid {:?}]:(`{:?}`)", sending_user_id, message);
        let request = MatchLobbySendMessageRequest::new(
            session_instance.instance_secret(),
            sending_user_id,
            message.to_string(),
        );

        let host = "session";
        let remote = "social";
        bevy_http_client::log_util::send_req(host, remote, MatchLobbySendMessageRequest::name());
        let response_key = http_client.send(social_server_addr, *social_server_port, request);

        self.in_flight_requests.push(ChatMessageReqInFlight::LobbyChatSendMessage(
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
        main_menu_room_key: &RoomKey,
        message_id: &MessageId,
        timestamp: &Timestamp,
        sending_user_id: &UserId,
        message: &str,
    ) {
        // spawn message entity
        let global_chat_message_entity =
            commands.spawn_empty().enable_replication(naia_server).id();
        let mut global_chat_message = ChatMessage::new(*message_id, *timestamp, message);

        // add to global chat room
        naia_server
            .room_mut(&main_menu_room_key)
            .add_entity(&global_chat_message_entity);

        // add to local log
        self.global_chat_message_entities
            .push_back(global_chat_message_entity);

        // remove oldest messages if we have too many
        if self.global_chat_message_entities.len() > 100 {
            let entity_to_delete = self.global_chat_message_entities.pop_front().unwrap();
            commands.entity(entity_to_delete).despawn();
        }

        let user_entity = user_manager.get_or_init_user_entity(commands, naia_server, http_client, main_menu_room_key, sending_user_id);

        global_chat_message
            .owner_user_entity
            .set(naia_server, &user_entity);
        commands
            .entity(global_chat_message_entity)
            .insert(global_chat_message)
            .insert(ChatMessageGlobal::new());
    }

    fn log_lobby_chat_message(
        &mut self,
        commands: &mut Commands,
        naia_server: &mut Server,
        http_client: &mut HttpClient,
        user_manager: &mut UserManager,
        lobby_manager: &LobbyManager,
        main_menu_room_key: &RoomKey,
        message_id: &MessageId,
        timestamp: &Timestamp,
        sending_user_id: &UserId,
        message: &str,
    ) {
        let lobby_id = user_manager.get_user_lobby_id(sending_user_id).unwrap();
        let lobby_room_key = lobby_manager.get_lobby_room_key(&lobby_id).unwrap();
        let lobby_entity = lobby_manager.get_lobby_entity(&lobby_id).unwrap();

        // spawn message entity
        let lobby_chat_message_entity =
            commands.spawn_empty().enable_replication(naia_server).id();

        // add to lobby room
        naia_server
            .room_mut(&lobby_room_key)
            .add_entity(&lobby_chat_message_entity);

        // add to local log
        if !self.lobby_chat_message_entities.contains_key(&lobby_id) {
            self.lobby_chat_message_entities.insert(lobby_id, VecDeque::new());
        }
        let lobby_chat_message_entities = self.lobby_chat_message_entities.get_mut(&lobby_id).unwrap();
        lobby_chat_message_entities.push_back(lobby_chat_message_entity);

        // remove oldest messages if we have too many
        if lobby_chat_message_entities.len() > 100 {
            let entity_to_delete = lobby_chat_message_entities.pop_front().unwrap();
            commands.entity(entity_to_delete).despawn();
        }

        let user_entity = user_manager.get_or_init_user_entity(commands, naia_server, http_client, main_menu_room_key, sending_user_id);

        let mut lobby_chat_message = ChatMessage::new(*message_id, *timestamp, message);
        lobby_chat_message
            .owner_user_entity
            .set(naia_server, &user_entity);

        let mut lobby_chat_message_local = ChatMessageLocal::new();
        lobby_chat_message_local.lobby_entity.set(naia_server, &lobby_entity);

        commands
            .entity(lobby_chat_message_entity)
            .insert(lobby_chat_message)
            .insert(lobby_chat_message_local);
    }

    pub(crate) fn patch_global_chat_messages(
        &mut self,
        commands: &mut Commands,
        naia_server: &mut Server,
        http_client: &mut HttpClient,
        user_manager: &mut UserManager,
        main_menu_room_key: &RoomKey,
        new_messages: &Vec<(MessageId, Timestamp, UserId, String)>,
    ) {
        for (msg_id, timestamp, user_id, message) in new_messages {
            // log the message
            self.log_global_chat_message(
                commands,
                naia_server,
                http_client,
                user_manager,
                main_menu_room_key,
                msg_id,
                timestamp,
                user_id,
                message,
            );
        }
    }

    pub(crate) fn patch_lobby_chat_message(
        &mut self,
        commands: &mut Commands,
        naia_server: &mut Server,
        http_client: &mut HttpClient,
        user_manager: &mut UserManager,
        lobby_manager: &LobbyManager,
        main_menu_room_key: &RoomKey,
        message_id: &MessageId,
        timestamp: &Timestamp,
        sending_user_id: &UserId,
        message: &str,
    ) {
        self.log_lobby_chat_message(
            commands,
            naia_server,
            http_client,
            user_manager,
            lobby_manager,
            main_menu_room_key,
            message_id,
            timestamp,
            sending_user_id,
            message,
        );
    }
}
