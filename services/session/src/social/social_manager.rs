use bevy_ecs::{system::Resource, change_detection::ResMut};

use auth_server_types::UserId;
use bevy_http_client::{HttpClient, ResponseKey};
use logging::{info, warn};

use social_server_http_proto::{GlobalChatSendMessageRequest, GlobalChatSendMessageResponse};

enum QueuedRequest {
    // session secret, user id, message
    GlobalChatSendMessage(String, UserId, String),
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
        mut social_manager: ResMut<Self>,
        mut http_client: ResMut<HttpClient>,
    ) {
        social_manager.process_in_flight_requests(
            &mut http_client,
        );
        social_manager.process_queued_requests(
            &mut http_client,
        );
    }

    pub fn process_queued_requests(
        &mut self,
        http_client: &mut HttpClient,
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
                QueuedRequest::GlobalChatSendMessage(session_secret, user_id, message) => {
                    self.send_global_chat_message(http_client, &session_secret, user_id, &message);
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
        http_client: &mut HttpClient,
        session_secret: &str,
        user_id: UserId,
        message: &str,
    ) {
        let Some((social_server_addr, social_server_port)) = self.get_social_server_url() else {
            warn!("received global chat message but no social server is available!");

            let qr = QueuedRequest::GlobalChatSendMessage(session_secret.to_string(), user_id, message.to_string());
            self.queued_requests.push(qr);

            return;
        };

        info!("sending global chat send message request to social server - [userid {:?}]:(`{:?}`)", user_id, message);
        let request = GlobalChatSendMessageRequest::new(session_secret, user_id, message);
        let response_key = http_client.send(&social_server_addr, social_server_port, request);

        let ifr = InFlightRequest::GlobalChatSendMessage(response_key);
        self.in_flight_requests.push(ifr);
    }
}
