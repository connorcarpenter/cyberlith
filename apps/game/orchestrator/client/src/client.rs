use std::collections::HashSet;

use bevy_ecs::{change_detection::ResMut, system::{SystemParam, Resource}};

use http::{HttpClient, HttpKey, HttpRequest, HttpResponse, HttpResponseError};

// Key
#[derive(Hash, PartialEq, Eq, Clone, Copy)]
pub struct OrchestratorRequestKey(pub(crate) HttpKey);

// State
#[derive(Resource)]
pub struct OrchestratorClientState {
    key_store: HashSet<HttpKey>,
}

impl Default for OrchestratorClientState {
    fn default() -> Self {
        Self {
            key_store: HashSet::new(),
        }
    }
}

// Param
#[derive(SystemParam)]
pub struct OrchestratorClient<'w> {
    http_client: ResMut<'w, HttpClient>,
    state: ResMut<'w, OrchestratorClientState>
}

impl<'w> OrchestratorClient<'w> {
    pub fn login(&mut self, username: &str, password: &str) -> OrchestratorRequestKey {
        let key = self.http_client.send(HttpRequest::get("https://api.ipify.org?format=json"));
        self.state.key_store.insert(key);
        OrchestratorRequestKey(key)
    }

    pub fn recv(&mut self, key: &OrchestratorRequestKey) -> Option<Result<HttpResponse, HttpResponseError>> {
        if let Some(result) = self.http_client.recv(&key.0) {
            self.state.key_store.remove(&key.0);
            Some(result)
        } else {
            None
        }
    }
}

//// send
//     if timer.0.ringing() {
//         timer.0.reset();
//
//         let key = http_client.send(HttpRequest::get("https://api.ipify.org?format=json"));
//         key_store.insert(key);
//     }
//
//     // recv
//     let mut received_keys = Vec::new();
//     for key in key_store.iter() {
//         if let Some(result) = http_client.recv(key) {
//             match result {
//                 Ok(response) => {
//                     let Some(text) = response.text() else {
//                         panic!("no text in response");
//                     };
//                     info!("response: {:?}", text);
//                 }
//                 Err(error) => {
//                     info!("error: {:?}", error);
//                 }
//             }
//
//             received_keys.push(*key);
//         }
//     }
//
//
//     // recv all
//     for (key, result) in http_client.recv_all() {
//         match result {
//             Ok(response) => {
//                 let Some(text) = response.text() else {
//                     panic!("no text in response");
//                 };
//                 info!("uncaught response: {:?}", text);
//             }
//             Err(error) => {
//                 info!("uncaught error: {:?}", error);
//
//             }
//         }
//
//         received_keys.push(key);
//     }
//
//     // remove received keys from list
//     for key in received_keys {
//         key_store.remove(&key);
//     }