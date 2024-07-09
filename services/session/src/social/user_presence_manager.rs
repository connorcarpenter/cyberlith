
use bevy_ecs::{
    system::{Commands, Query},
};

use naia_bevy_server::{RoomKey, Server};

use bevy_http_client::{ApiRequest, ApiResponse, HttpClient, ResponseKey};
use logging::{info, warn};

use auth_server_types::UserId;

use session_server_http_proto::SocialUserPatch;
use session_server_naia_proto::components::{PublicUserInfo};

use social_server_http_proto::{UserDisconnectedRequest, UserDisconnectedResponse};

use crate::{session_instance::SessionInstance, user::UserManager};

struct UserSendDisconnectQueued(UserId);
struct UserSendDisconnectInFlight(UserId, ResponseKey<UserDisconnectedResponse>);

pub struct UserPresenceManager {
    queued_requests: Vec<UserSendDisconnectQueued>,
    in_flight_requests: Vec<UserSendDisconnectInFlight>,
}

impl UserPresenceManager {
    pub fn new() -> Self {
        Self {
            queued_requests: Vec::new(),
            in_flight_requests: Vec::new(),
        }
    }

    pub(crate) fn update(
        &mut self,
        http_client: &mut HttpClient,
        social_server_url: &Option<(String, u16)>,
        session_instance: &SessionInstance,
        user_manager: &mut UserManager,
        users_q: &mut Query<&mut PublicUserInfo>,
    ) {
        self.process_in_flight_requests(http_client, user_manager, users_q);
        self.process_queued_requests(http_client, social_server_url, session_instance);
    }

    pub(crate) fn patch_users(
        &mut self,
        commands: &mut Commands,
        naia_server: &mut Server,
        http_client: &mut HttpClient,
        user_manager: &mut UserManager,
        global_chat_room_key: &RoomKey,
        users_q: &mut Query<&mut PublicUserInfo>,
        user_patches: &Vec<SocialUserPatch>,
    ) {
        for user_patch in user_patches {
            match user_patch {
                SocialUserPatch::Add(user_id) => {
                    info!("adding user - [userid {:?}]", user_id);

                    if !user_manager.has_user_data(user_id) {
                        user_manager.add_user_data(
                            commands,
                            naia_server,
                            http_client,
                            global_chat_room_key,
                            user_id,
                        );
                    }

                    user_manager.user_set_online(user_id, users_q);
                }
                SocialUserPatch::Remove(user_id) => {
                    info!("removing user - [userid {:?}]", user_id);

                    user_manager.user_set_offline(user_id, users_q);
                }
            }
        }
    }

    pub fn send_user_disconnect_req(
        &mut self,
        http_client: &mut HttpClient,
        social_server_url: Option<&(String, u16)>,
        session_instance: &SessionInstance,
        user_id: &UserId,
    ) {
        let Some((social_server_addr, social_server_port)) = social_server_url else {
            warn!("user disconnected but no social server is available!");

            self.queued_requests.push(UserSendDisconnectQueued(*user_id));

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
        let response_key = http_client.send(social_server_addr, *social_server_port, request);

        self.in_flight_requests.push(UserSendDisconnectInFlight(*user_id, response_key));

        return;
    }


    pub fn process_queued_requests(
        &mut self,
        http_client: &mut HttpClient,
        social_server_url: &Option<(String, u16)>,
        session_instance: &SessionInstance,
    ) {
        if self.queued_requests.is_empty() {
            // no queued assets
            return;
        }
        if social_server_url.is_none() {
            // it's okay to wait until the social server is available
            return;
        };

        let queued_requests = std::mem::take(&mut self.queued_requests);
        for request in queued_requests {
            let user_id = request.0;
            self.send_user_disconnect_req(
                http_client,
                social_server_url.as_ref(),
                session_instance,
                &user_id
            );
        }
    }

    pub fn process_in_flight_requests(
        &mut self,
        http_client: &mut HttpClient,
        user_manager: &mut UserManager,
        users_q: &mut Query<&mut PublicUserInfo>,
    ) {
        if self.in_flight_requests.is_empty() {
            // no in-flight requests
            return;
        }

        let mut continuing_requests = Vec::new();
        let in_flight_requests = std::mem::take(&mut self.in_flight_requests);

        for req in in_flight_requests {
            let UserSendDisconnectInFlight(user_id, response_key) = &req;

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
                            users_q,
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

        self.in_flight_requests = continuing_requests;
    }
}
