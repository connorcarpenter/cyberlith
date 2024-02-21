
use std::collections::HashSet;

use bevy_log::info;

use naia_bevy_server::{ResponseReceiveKey, Server, UserKey};

use asset_server_http_proto::{AssetRequest, AssetResponse, AssetResponseValue};
use session_server_naia_proto::{channels::{PrimaryChannel, RequestChannel}, messages::{LoadAssetRequest, LoadAssetResponse, LoadAssetWithData, LoadAssetResponseValue}};

use asset_id::{AssetId, ETag};
use bevy_http_client::{HttpClient, ResponseKey};

use crate::asset::asset_store::AssetStore;

struct AssetServerRequestState {
    asset_server_request: AssetRequest,
    asset_server_response_key: Option<ResponseKey<AssetResponse>>,
    asset_server_response: Option<AssetResponse>,
}

impl AssetServerRequestState {
    pub fn new(
        asset_server_request: AssetRequest,
        asset_server_response_key: ResponseKey<AssetResponse>,
    ) -> Self {
        Self {
            asset_server_request,
            asset_server_response_key: Some(asset_server_response_key),
            asset_server_response: None,
        }
    }

    pub fn process(&mut self, http_client: &mut HttpClient) {
        if let Some(key) = self.asset_server_response_key.as_ref() {
            if let Some(response_result) = http_client.recv(key) {
                match response_result {
                    Ok(response) => {
                        info!("received asset response from asset server");
                        self.asset_server_response_key = None;
                        self.asset_server_response = Some(response);
                    }
                    Err(e) => {
                        panic!("error receiving asset response: {:?}", e.to_string());
                    }
                }
            }
        }
    }

    pub fn completed(&self) -> bool {
        self.asset_server_response.is_some()
    }

    pub fn unwrap_response(self) -> (AssetRequest, AssetResponse) {
        (
            self.asset_server_request,
            self.asset_server_response.unwrap(),
        )
    }
}

struct ClientLoadAssetRequestState {
    load_asset_request: LoadAssetRequest,
    load_asset_response_key: Option<ResponseReceiveKey<LoadAssetResponse>>,
    load_asset_response: Option<LoadAssetResponse>,
}

impl ClientLoadAssetRequestState {
    pub fn new(
        load_asset_request: LoadAssetRequest,
        load_asset_response_key: ResponseReceiveKey<LoadAssetResponse>,
    ) -> Self {
        Self {
            load_asset_request,
            load_asset_response_key: Some(load_asset_response_key),
            load_asset_response: None,
        }
    }

    pub fn process(&mut self, server: &mut Server) {
        if let Some(key) = self.load_asset_response_key.as_ref() {
            if let Some((_user_key, response)) = server.receive_response(key) {
                info!("received asset etag response from client");
                self.load_asset_response_key = None;
                self.load_asset_response = Some(response);
            }
        }
    }

    pub fn completed(&self) -> bool {
        self.load_asset_response.is_some()
    }

    pub fn unwrap_response(self) -> (LoadAssetRequest, LoadAssetResponse) {
        (self.load_asset_request, self.load_asset_response.unwrap())
    }
}

pub struct UserAssets {
    user_key: UserKey,
    assets_in_memory: HashSet<AssetId>,
    asset_server_requests: Vec<AssetServerRequestState>,
    client_load_asset_requests: Vec<ClientLoadAssetRequestState>,
}

impl UserAssets {
    pub fn new(user_key: &UserKey) -> Self {
        Self {
            user_key: *user_key,
            assets_in_memory: HashSet::new(),
            asset_server_requests: Vec::new(),
            client_load_asset_requests: Vec::new(),
        }
    }

    pub fn user_asset_request(
        &mut self,
        http_client: &mut HttpClient,
        asset_server_addr: &str,
        asset_server_port: u16,
        asset_store: &AssetStore,
        asset_id: &AssetId,
        added: bool,
    ) {
        if added {
            self.user_asset_added(
                http_client,
                asset_server_addr,
                asset_server_port,
                asset_store,
                asset_id,
            );
        } else {
            self.user_asset_removed(asset_id);
        }
    }

    pub fn process_asset_server_requests(
        &mut self,
        http_client: &mut HttpClient,
    ) -> Option<Vec<(AssetId, ETag, Option<(HashSet<AssetId>, Vec<u8>)>)>> {
        let asset_server_requests = std::mem::take(&mut self.asset_server_requests);

        if asset_server_requests.is_empty() {
            return None;
        }

        let mut asset_server_responses = Vec::new();

        for mut request in asset_server_requests {
            request.process(http_client);
            if request.completed() {
                asset_server_responses.push(request.unwrap_response());
            } else {
                self.asset_server_requests.push(request);
            }
        }

        if asset_server_responses.is_empty() {
            return None;
        }

        let mut asset_server_responses = Vec::new();
        for (asset_server_req, asset_res) in asset_server_responses {

            let asset_id = asset_server_req.asset_id();
            let old_etag_opt = asset_server_req.etag_opt();

            match asset_res.value {
                AssetResponseValue::Modified(new_etag, dependencies, data) => {

                    info!("asset server responded with new etag: {:?}. storing asset data for: {:?}", new_etag, asset_id);

                    // process dependencies
                    let mut dependency_set = HashSet::new();
                    for dependency in dependencies {
                        dependency_set.insert(dependency);
                    }

                    // store new asset etag & data
                    asset_server_responses.push((asset_id, new_etag, Some((dependency_set, data))));
                }
                AssetResponseValue::NotModified => {
                    info!("asset server responded with data not modified, storing asset data for: {:?}", asset_id);

                    asset_server_responses.push((asset_id, old_etag_opt.unwrap(), None));
                }
            }
        }

        let asset_server_responses = if asset_server_responses.is_empty() {
            None
        } else {
            Some(asset_server_responses)
        };
        asset_server_responses
    }

    pub fn process_client_load_asset_requests(
        &mut self,
        server: &mut Server,
    ) -> Option<Vec<(UserKey, AssetId)>> {
        let client_load_asset_requests = std::mem::take(&mut self.client_load_asset_requests);

        if client_load_asset_requests.is_empty() {
            return None;
        }

        let mut client_load_asset_responses = Vec::new();

        for mut request in client_load_asset_requests {
            request.process(server);
            if request.completed() {
                client_load_asset_responses.push(request.unwrap_response());
            } else {
                self.client_load_asset_requests.push(request);
            }
        }

        if client_load_asset_responses.is_empty() {
            return None;
        }

        let mut pending_client_requests = Vec::new();
        for (client_etag_req, client_etag_res) in client_load_asset_responses {

            let asset_id = client_etag_req.asset_id;

            match client_etag_res.value {
                LoadAssetResponseValue::ClientHasOldOrNoAsset => {
                    info!(
                        "client responded with old data for asset {:?}, sending new asset data to client.",
                        asset_id
                    );

                    pending_client_requests.push((self.user_key, asset_id));
                }
                LoadAssetResponseValue::ClientLoadedNonModifiedAsset => {
                    info!("client already has latest data for asset: {:?}", asset_id);

                    self.assets_in_memory.insert(asset_id);
                }
            }
        }

        let pending_client_requests = if pending_client_requests.is_empty() {
            None
        } else {
            Some(pending_client_requests)
        };
        pending_client_requests
    }

    pub(crate) fn send_client_asset_data(
        &mut self,
        server: &mut Server,
        asset_store: &AssetStore,
        asset_id: &AssetId,
    ) {
        info!("sending asset data to client: {:?}", asset_id);

        // get asset etag & data from store
        let (etag, data) = asset_store.get_etag_and_data(asset_id).unwrap();
        let message = LoadAssetWithData::new(*asset_id, etag, data);
        server.send_message::<PrimaryChannel, _>(&self.user_key, &message);

        // mark asset as in memory
        self.assets_in_memory.insert(*asset_id);
    }

    fn user_asset_added(
        &mut self,
        http_client: &mut HttpClient,
        asset_server_addr: &str,
        asset_server_port: u16,
        asset_store: &AssetStore,
        asset_id: &AssetId,
    ) {
        // does user already have the asset in memory? if so, return
        if self.assets_in_memory.contains(asset_id) {
            return;
        }

        // send 'asset' request to asset server
        info!("sending asset request to asset server: {:?}", asset_id);
        let etag_opt = asset_store.get_etag(asset_id);
        let asset_server_request = AssetRequest::new(*asset_id, etag_opt);
        let asset_server_response_key = http_client.send(
            asset_server_addr,
            asset_server_port,
            asset_server_request.clone(),
        );

        // save responsekeys for 'load_asset' and 'load_asset_with_data' requests
        self.asset_server_requests.push(AssetServerRequestState::new(
            asset_server_request,
            asset_server_response_key,
        ));
    }

    pub fn send_client_load_asset_request(&mut self, server: &mut Server, asset_id: &AssetId, etag: &ETag) {
        // send 'load_asset' request to client
        info!("sending load_asset request to client: (asset: {:?}, etag: {:?})", asset_id, etag);
        let asset_etag_request = LoadAssetRequest::new(asset_id, etag);
        let asset_etag_response_key = server
            .send_request::<RequestChannel, _>(&self.user_key, &asset_etag_request)
            .unwrap();

        // save responsekeys for 'load_asset' requests
        self.client_load_asset_requests.push(ClientLoadAssetRequestState::new(
            asset_etag_request,
            asset_etag_response_key,
        ));
    }

    fn user_asset_removed(&mut self, _asset_id: &AssetId) {
        todo!()
    }
}
