use std::{net::SocketAddr};
use std::any::TypeId;
use std::collections::HashMap;

use async_dup::Arc;
use bevy_ecs::{change_detection::ResMut, system::Resource};
use log::warn;

use smol::channel::{Receiver, Sender};
use smol::lock::RwLock;

use bevy_http_shared::Protocol;
use http_common::{ApiRequest, ApiResponse, Request, Response};

use crate::{ResponseKey, server_state::ServerState};

#[derive(Resource)]
pub struct HttpServer {
    state: Option<ServerState>,
    request_receivers: HashMap<TypeId, Receiver<(u64, SocketAddr, Request)>>,
    response_sender: Sender<(u64, Response)>,
    listening: bool,
}

impl HttpServer {

    pub fn new(protocol: Protocol) -> Self {
        let (state, request_receivers, response_sender) = ServerState::new(protocol);

        Self {
            state: Some(state),
            request_receivers,
            response_sender,
            listening: false,
        }
    }

    pub fn listen(&mut self, addr: SocketAddr) {
        if self.listening {
            panic!("already listening!");
        }
        let state = self.state.take().unwrap();
        ServerState::listen(state, addr);
        self.listening = true;
    }

    pub fn receive<Q: ApiRequest>(&mut self) -> Option<(SocketAddr, Q, ResponseKey<Q::Response>)> {
        let Some(request_receiver) = self.request_receivers.get(&TypeId::of::<Q>()) else {
            panic!("did not register type!");
        };
        if let Ok((response_id, request_addr, request)) = request_receiver.try_recv() {

            let Ok(request) = Q::from_request(request) else {
                warn!("could not deserialize request");
                return None;
            };
            let response_key = ResponseKey::new(response_id);
            Some((request_addr, request, response_key))
        } else {
            None
        }
    }

    pub fn respond<S: ApiResponse>(&mut self, key: ResponseKey<S>, response: S) {
        let id = key.id;
        let response = response.to_response();
        self.response_sender.try_send((id, response)).unwrap();
    }
}