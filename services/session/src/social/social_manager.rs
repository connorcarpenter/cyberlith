use bevy_ecs::{system::{Resource, Res}, change_detection::ResMut};

use naia_bevy_server::{Server, UserKey};

use bevy_http_client::{HttpClient, ResponseKey};
use logging::{info, warn};

use session_server_naia_proto::{channels::PrimaryChannel, messages::GlobalChatRecvMessage};
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
}

impl SocialManager {
    pub fn new() -> Self {
        Self {
            social_server_opt: None,
            in_flight_requests: Vec::new(),
            queued_requests: Vec::new(),
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
    pub fn update(
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
            &mut naia_server, &mut http_client, &session_instance, &user_manager,
        );
    }

    pub fn process_queued_requests(
        &mut self,
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

        // send messages to all other connected users
        for user_key in &naia_server.user_keys() {
            if user_key == sending_user_key {
                continue;
            }

            let message_to_user = GlobalChatRecvMessage::new(sending_user_id, message);
            naia_server.send_message::<PrimaryChannel, _>(&user_key, &message_to_user);
        }
    }
}
