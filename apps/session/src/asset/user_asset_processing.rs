
use std::collections::HashSet;

use bevy_log::info;

use naia_bevy_server::{ResponseReceiveKey, Server};

use asset_server_http_proto::{AssetRequest, AssetResponse, AssetResponseValue};
use session_server_naia_proto::{messages::{LoadAssetResponse, LoadAssetResponseValue}};

use asset_id::{AssetId, ETag};
use bevy_http_client::{HttpClient, ResponseKey};

// UserAssetProcessingState
pub enum UserAssetProcessingState {
    AssetServerRequestInFlight(AssetServerRequestState),
    ClientLoadAssetRequestInFlight(ClientLoadAssetRequestState),
}

impl UserAssetProcessingState {

    pub fn asset_server_request_in_flight(
        request: AssetRequest,
        response_key: ResponseKey<AssetResponse>
    ) -> Self {
        Self::AssetServerRequestInFlight(AssetServerRequestState::new(request, response_key))
    }

    pub fn client_load_asset_request_in_flight(
        response_key: ResponseReceiveKey<LoadAssetResponse>
    ) -> Self {
        Self::ClientLoadAssetRequestInFlight(ClientLoadAssetRequestState::new(response_key))
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