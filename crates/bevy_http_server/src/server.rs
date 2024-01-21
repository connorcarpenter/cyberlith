use std::net::SocketAddr;

use bevy_ecs::{change_detection::ResMut, system::Resource};

use http_common::{ApiRequest, ApiResponse};

use crate::ResponseKey;

#[derive(Resource)]
pub struct HttpServer {

}

impl Default for HttpServer {
    fn default() -> Self {
        Self {

        }
    }
}

impl HttpServer {

    pub fn listen(&mut self, addr: SocketAddr) {

    }

    pub fn receive<Q: ApiRequest>(&mut self) -> Option<(SocketAddr, Q, ResponseKey<Q::Response>)> {
        None
    }

    pub fn respond<S: ApiResponse>(&mut self, key: ResponseKey<S>, response: S) {

    }
}

pub(crate) fn server_update(mut server: ResMut<HttpServer>) {

}
