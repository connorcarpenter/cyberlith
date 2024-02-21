
use std::collections::HashSet;

use bevy_log::info;

use naia_bevy_server::{ResponseReceiveKey, Server, UserKey};

use asset_server_http_proto::{AssetRequest, AssetResponse, AssetResponseValue};
use session_server_naia_proto::{channels::RequestChannel, messages::{LoadAssetResponse, LoadAssetRequest, LoadAssetResponseValue}};

use asset_id::{AssetId, ETag};
use bevy_http_client::{HttpClient, ResponseKey};

// UserAssetProcessingState
pub enum UserAssetProcessingState {
    AssetServerRequestInFlight(AssetServerRequestState),
    WaitingForDependencies(HashSet<AssetId>),
    ClientLoadAssetRequestInFlight(ClientLoadAssetRequestState),
}

impl UserAssetProcessingState {

    pub fn send_asset_server_request(
        http_client: &mut HttpClient,
        asset_server_addr: &str,
        asset_server_port: u16,
        asset_id: &AssetId,
        asset_etag_opt: Option<ETag>,
    ) -> Self {
        info!("sending asset request to asset server: {:?}", asset_id);
        let request = AssetRequest::new(*asset_id, asset_etag_opt);
        let response_key = http_client.send(
            asset_server_addr,
            asset_server_port,
            request.clone(),
        );

        Self::AssetServerRequestInFlight(AssetServerRequestState::new(request, response_key))
    }

    pub fn send_client_load_asset_request(
        server: &mut Server,
        user_key: &UserKey,
        asset_id: &AssetId,
        asset_etag: &ETag,
    ) -> Self {

        // send client "load asset" request
        info!("sending load_asset request to client: (asset: {:?}, etag: {:?})", asset_id, asset_etag);
        let request = LoadAssetRequest::new(asset_id, asset_etag);
        let response_key = server
            .send_request::<RequestChannel, _>(user_key, &request)
            .unwrap();

        Self::ClientLoadAssetRequestInFlight(ClientLoadAssetRequestState::new(response_key))
    }

    pub fn waiting_for_dependencies(dependencies: HashSet<AssetId>) -> Self {
        Self::WaitingForDependencies(dependencies)
    }

    // returns whether all dependencies have loaded
    pub(crate) fn handle_dependency_loaded(&mut self, dependency: &AssetId) -> bool {
        let UserAssetProcessingState::WaitingForDependencies(dependencies) = self else {
            panic!("handle_dependency_loaded called on non-WaitingForDependencies state");
        };
        let removal_result = dependencies.remove(dependency);
        if !removal_result {
            panic!("dependency not found in waiting list");
        }
        return dependencies.is_empty();
    }

    pub(crate) fn process(
        &mut self,
        server: &mut Server,
        http_client: &mut HttpClient,
    ) -> Option<UserAssetProcessingStateTransition> {

        match self {
            UserAssetProcessingState::AssetServerRequestInFlight(state) => {
                return state.process(http_client);
            }
            UserAssetProcessingState::ClientLoadAssetRequestInFlight(state) => {
                return state.process(server);
            }
            UserAssetProcessingState::WaitingForDependencies(_) => {
                // do nothing
                return None;
            }
        }
    }
}

// AssetServerRequestState
pub struct AssetServerRequestState {
    request: AssetRequest,
    response_key: Option<ResponseKey<AssetResponse>>,
    response: Option<AssetResponse>,
}

impl AssetServerRequestState {
    pub fn new(
        request: AssetRequest,
        response_key: ResponseKey<AssetResponse>,
    ) -> Self {
        Self {
            request,
            response_key: Some(response_key),
            response: None,
        }
    }

    pub(crate) fn process(&mut self, http_client: &mut HttpClient) -> Option<UserAssetProcessingStateTransition> {

        if let Some(key) = self.response_key.as_ref() {
            if let Some(response_result) = http_client.recv(key) {
                match response_result {
                    Ok(response) => {
                        info!("received asset response from asset server");
                        self.response_key = None;
                        self.response = Some(response);
                    }
                    Err(e) => {
                        panic!("error receiving asset response: {:?}", e.to_string());
                    }
                }
            } else {
                // still waiting for response
                return None;
            }
        } else {
            panic!("process_in_flight_requests called on completed AssetServerRequestState");
        }

        // response received!

        let asset_server_req = &self.request;
        let asset_server_res = self.response.as_ref().unwrap();

        let asset_id = asset_server_req.asset_id();
        let old_etag_opt = asset_server_req.etag_opt();

        match &asset_server_res.value {
            AssetResponseValue::Modified(new_etag, dependencies, data) => {

                info!("asset server responded with new etag: {:?}. storing asset data for: {:?}", new_etag, asset_id);

                // process dependencies
                let mut dependency_set = HashSet::new();
                for dependency in dependencies {
                    dependency_set.insert(*dependency);
                }

                // store new asset etag & data
                return Some(UserAssetProcessingStateTransition::asset_server_response(*new_etag, Some((dependency_set, data.clone()))));
            }
            AssetResponseValue::NotModified => {
                info!("asset server responded with data not modified, storing asset data for: {:?}", asset_id);

                return Some(UserAssetProcessingStateTransition::asset_server_response(old_etag_opt.unwrap(), None));
            }
        }
    }
}

// ClientLoadAssetRequestState
pub struct ClientLoadAssetRequestState {
    response_key: Option<ResponseReceiveKey<LoadAssetResponse>>,
    response: Option<LoadAssetResponse>,
}

impl ClientLoadAssetRequestState {
    pub fn new(
        response_key: ResponseReceiveKey<LoadAssetResponse>,
    ) -> Self {
        Self {
            response_key: Some(response_key),
            response: None,
        }
    }

    pub fn process(&mut self, server: &mut Server) -> Option<UserAssetProcessingStateTransition> {

        if let Some(key) = self.response_key.as_ref() {
            if let Some((_user_key, response)) = server.receive_response(key) {
                info!("received asset etag response from client");
                self.response_key = None;
                self.response = Some(response);
            } else {
                // still waiting for response
                return None;
            }
        } else {
            panic!("process_in_flight_requests called on completed ClientLoadAssetRequestState");
        }

        // response received
        let response = self.response.as_ref().unwrap();

        return Some(UserAssetProcessingStateTransition::client_load_asset_response(response.value));
    }
}

// UserAssetProcessingStateTransition
pub enum UserAssetProcessingStateTransition {
    AssetServerResponse(ETag, Option<(HashSet<AssetId>, Vec<u8>)>),
    ClientLoadAssetResponse(LoadAssetResponseValue),
}

impl UserAssetProcessingStateTransition {
    pub fn asset_server_response(etag: ETag, data_opt: Option<(HashSet<AssetId>, Vec<u8>)>) -> Self {
        Self::AssetServerResponse(etag, data_opt)
    }

    pub fn client_load_asset_response(response_value: LoadAssetResponseValue) -> Self {
        Self::ClientLoadAssetResponse(response_value)
    }
}