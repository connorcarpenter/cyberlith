use std::collections::HashMap;

use auth_server_http_proto::{UserGetRequest, UserGetResponse};
use auth_server_types::UserId;
use bevy_http_client::{ApiRequest, ApiResponse, HttpClient, ResponseKey};
use config::{AUTH_SERVER_PORT, AUTH_SERVER_RECV_ADDR};
use logging::{info, warn};

pub(crate) struct UserInfoService {
    inflight_user_info_requests: HashMap<UserId, ResponseKey<UserGetResponse>>,
}

impl UserInfoService {
    pub fn new() -> Self {
        Self {
            inflight_user_info_requests: HashMap::new(),
        }
    }

    pub fn process_in_flight_requests(
        &mut self,
        http_client: &mut HttpClient,
    ) -> Option<Vec<(UserId, UserGetResponse)>> {
        if self.inflight_user_info_requests.is_empty() {
            // no in-flight requests
            return None;
        }

        let mut received_responses = Vec::new();
        for (nameless_user_id, response_key) in self.inflight_user_info_requests.iter() {
            let nameless_user_id = *nameless_user_id;

            if let Some(response_result) = http_client.recv(response_key) {
                let host = "session";
                let remote = "auth";
                bevy_http_client::log_util::recv_res(host, remote, UserGetResponse::name());

                match response_result {
                    Ok(response) => {
                        info!("received user get response from auth server");
                        received_responses.push((nameless_user_id, response));
                    }
                    Err(e) => {
                        warn!(
                            "error receiving user get response from social server: {:?}",
                            e.to_string()
                        );
                    }
                }
            }
        }

        let mut output = Vec::new();
        for (user_id, received_response) in received_responses {
            self.inflight_user_info_requests.remove(&user_id);

            output.push((user_id, received_response));
        }

        Some(output)
    }

    pub fn send_user_info_request(&mut self, http_client: &mut HttpClient, user_id: &UserId) {
        let request = UserGetRequest::new(*user_id);

        let host = "session";
        let remote = "auth";
        bevy_http_client::log_util::send_req(host, remote, UserGetRequest::name());
        let response_key = http_client.send(AUTH_SERVER_RECV_ADDR, AUTH_SERVER_PORT, request);

        self.inflight_user_info_requests
            .insert(*user_id, response_key);
    }
}
