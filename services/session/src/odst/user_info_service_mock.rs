use std::collections::HashSet;

use auth_server_http_proto::UserGetResponse;
use auth_server_types::{UserId, UserRole};
use bevy_http_client::HttpClient;

pub(crate) struct UserInfoService {
    inflight_user_info_requests: HashSet<UserId>,
}

impl UserInfoService {
    pub fn new() -> Self {
        Self {
            inflight_user_info_requests: HashSet::new(),
        }
    }

    pub fn process_in_flight_requests(
        &mut self,
        _http_client: &mut HttpClient,
    ) -> Option<Vec<(UserId, UserGetResponse)>> {
        if self.inflight_user_info_requests.is_empty() {
            // no in-flight requests
            return None;
        }

        let mut received_responses = Vec::new();
        for nameless_user_id in self.inflight_user_info_requests.iter() {
            let nameless_user_id = *nameless_user_id;

            let response =
                UserGetResponse::new("connor".to_string(), "".to_string(), UserRole::Free);

            received_responses.push((nameless_user_id, response));
        }

        let mut output = Vec::new();
        for (user_id, received_response) in received_responses {
            self.inflight_user_info_requests.remove(&user_id);

            output.push((user_id, received_response));
        }

        Some(output)
    }

    pub fn send_user_info_request(&mut self, _http_client: &mut HttpClient, user_id: &UserId) {
        self.inflight_user_info_requests.insert(*user_id);
    }
}
