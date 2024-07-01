use std::collections::HashSet;

use naia_bevy_server::{ResponseReceiveKey, Server, UserKey};

use asset_id::{AssetId, AssetType, ETag};
use bevy_http_client::{ApiRequest, ApiResponse, HttpClient, ResponseKey};
use logging::info;

use asset_server_http_proto::{AssetRequest, AssetResponse, AssetResponseValue};
use session_server_naia_proto::{
    channels::AssetRequestsChannel,
    messages::{LoadAssetRequest, LoadAssetResponse, LoadAssetResponseValue},
};

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
        // info!("sending asset request to asset server: {:?}", asset_id);
        let request = AssetRequest::new(*asset_id, asset_etag_opt);

        let host = "session";
        let remote = "asset";
        let request_str = format!("{} ({:?})", AssetRequest::name(), asset_id);
        bevy_http_client::log_util::send_req(host, remote, &request_str);
        let response_key = http_client.send(asset_server_addr, asset_server_port, request.clone());

        Self::AssetServerRequestInFlight(AssetServerRequestState::new(request, response_key))
    }

    pub fn send_client_load_asset_request(
        server: &mut Server,
        user_key: &UserKey,
        asset_id: &AssetId,
        asset_etag: &ETag,
    ) -> Self {
        // send client "load asset" request
        // info!(
        //     "sending load_asset request to client: (asset: {:?}, etag: {:?})",
        //     asset_id, asset_etag
        // );
        let request = LoadAssetRequest::new(asset_id, asset_etag);

        let host = "session";
        let remote = "client";
        bevy_http_client::log_util::send_req(host, remote, LoadAssetRequest::name());
        let response_key = server
            .send_request::<AssetRequestsChannel, _>(user_key, &request)
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
    pub fn new(request: AssetRequest, response_key: ResponseKey<AssetResponse>) -> Self {
        Self {
            request,
            response_key: Some(response_key),
            response: None,
        }
    }

    pub(crate) fn process(
        &mut self,
        http_client: &mut HttpClient,
    ) -> Option<UserAssetProcessingStateTransition> {
        if let Some(key) = self.response_key.as_ref() {
            if let Some(response_result) = http_client.recv(key) {
                match response_result {
                    Ok(response) => {
                        self.response_key = None;
                        self.response = Some(response);
                    }
                    Err(e) => {
                        let asset_id = self.request.asset_id();
                        panic!(
                            "error receiving asset response for [asset {:?}] error: {:?}",
                            asset_id,
                            e.to_string()
                        );
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
            AssetResponseValue::Modified(new_etag, asset_type, dependencies, data) => {
                // info!("received from assetserver: asset_response(asset: {:?}, new etag: {:?}), storing data.", asset_id, new_etag);

                let host = "session";
                let remote = "asset";
                let response_str = format!(
                    "{} (modified, {:?}, {:?})",
                    AssetResponse::name(),
                    asset_id,
                    new_etag
                );
                bevy_http_client::log_util::recv_res(host, remote, &response_str);

                // process dependencies
                let mut dependency_set = HashSet::new();
                for dependency in dependencies {
                    dependency_set.insert(*dependency);
                }

                // store new asset etag & data
                return Some(UserAssetProcessingStateTransition::asset_server_response(
                    *new_etag,
                    Some((*asset_type, dependency_set, data.clone())),
                ));
            }
            AssetResponseValue::NotModified => {
                info!("received from assetserver: asset_response(asset: {:?}, with data not modified).", asset_id);

                let host = "session";
                let remote = "asset";
                let response_str = format!(
                    "{:?}[data not modified, asset_id: {:?}]",
                    AssetResponse::name(),
                    asset_id
                );
                bevy_http_client::log_util::recv_res(host, remote, &response_str);

                return Some(UserAssetProcessingStateTransition::asset_server_response(
                    old_etag_opt.unwrap(),
                    None,
                ));
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
    pub fn new(response_key: ResponseReceiveKey<LoadAssetResponse>) -> Self {
        Self {
            response_key: Some(response_key),
            response: None,
        }
    }

    pub fn process(&mut self, server: &mut Server) -> Option<UserAssetProcessingStateTransition> {
        if let Some(key) = self.response_key.as_ref() {
            if let Some((_user_key, response)) = server.receive_response(key) {
                let host = "session";
                let remote = "client";
                bevy_http_client::log_util::recv_res(host, remote, LoadAssetResponse::name());

                // info!("received 'load_asset' response from client");

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

        return Some(
            UserAssetProcessingStateTransition::client_load_asset_response(response.value),
        );
    }
}

// UserAssetProcessingStateTransition
pub enum UserAssetProcessingStateTransition {
    AssetServerResponse(ETag, Option<(AssetType, HashSet<AssetId>, Vec<u8>)>),
    ClientLoadAssetResponse(LoadAssetResponseValue),
}

impl UserAssetProcessingStateTransition {
    pub fn asset_server_response(
        etag: ETag,
        data_opt: Option<(AssetType, HashSet<AssetId>, Vec<u8>)>,
    ) -> Self {
        Self::AssetServerResponse(etag, data_opt)
    }

    pub fn client_load_asset_response(response_value: LoadAssetResponseValue) -> Self {
        Self::ClientLoadAssetResponse(response_value)
    }
}
