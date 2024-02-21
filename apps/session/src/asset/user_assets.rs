
use std::collections::{HashMap, HashSet};

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

    pub(crate) fn process(&mut self, http_client: &mut HttpClient) -> Option<UserAssetStateTransition> {

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
            } else {
                // still waiting for response
                return None;
            }
        } else {
            panic!("process_in_flight_requests called on completed AssetServerRequestState");
        }

        // response received!

        let asset_server_req = &self.asset_server_request;
        let asset_server_res = self.asset_server_response.as_ref().unwrap();

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
                return Some(UserAssetStateTransition::AssetServerResponse(*new_etag, Some((dependency_set, data.clone()))));
            }
            AssetResponseValue::NotModified => {
                info!("asset server responded with data not modified, storing asset data for: {:?}", asset_id);

                return Some(UserAssetStateTransition::AssetServerResponse(old_etag_opt.unwrap(), None));
            }
        }
    }
}

struct ClientLoadAssetRequestState {
    load_asset_response_key: Option<ResponseReceiveKey<LoadAssetResponse>>,
    load_asset_response: Option<LoadAssetResponse>,
}

impl ClientLoadAssetRequestState {
    pub fn new(
        load_asset_response_key: ResponseReceiveKey<LoadAssetResponse>,
    ) -> Self {
        Self {
            load_asset_response_key: Some(load_asset_response_key),
            load_asset_response: None,
        }
    }

    pub fn process(&mut self, server: &mut Server) -> Option<UserAssetStateTransition> {

        if let Some(key) = self.load_asset_response_key.as_ref() {
            if let Some((_user_key, response)) = server.receive_response(key) {
                info!("received asset etag response from client");
                self.load_asset_response_key = None;
                self.load_asset_response = Some(response);
            } else {
                // still waiting for response
                return None;
            }
        } else {
            panic!("process_in_flight_requests called on completed ClientLoadAssetRequestState");
        }

        // response received
        let response = self.load_asset_response.as_ref().unwrap();

        return Some(UserAssetStateTransition::ClientLoadAssetResponse(response.value));

        // {
        //     LoadAssetResponseValue::ClientHasOldOrNoAsset => {
        //         info!(
        //             "client responded with old data for asset {:?}, sending new asset data to client.",
        //             asset_id
        //         );
        //
        //         return Some(asset_id);
        //         pending_client_requests.push((self.user_key, asset_id));
        //     }
        //     LoadAssetResponseValue::ClientLoadedNonModifiedAsset => {
        //         info!("client already has latest data for asset: {:?}", asset_id);
        //
        //         self.assets_in_memory.insert(asset_id);
        //     }
        // }
        //
        //
        // // send pending client "load_asset_with_data" requests
        // for (user_key, asset_id) in pending_client_requests {
        //     self.send_client_asset_data(server, &user_key, &asset_id);
        // }
    }
}

enum UserAssetState {
    AssetServerRequestInFlight(AssetServerRequestState),
    ClientLoadAssetRequestInFlight(ClientLoadAssetRequestState),
}

enum UserAssetStateTransition {
    AssetServerResponse(ETag, Option<(HashSet<AssetId>, Vec<u8>)>),
    ClientLoadAssetResponse(LoadAssetResponseValue),
}

impl UserAssetState {
    pub(crate) fn process_in_flight_requests(
        &mut self,
        server: &mut Server,
        http_client: &mut HttpClient,
    ) -> Option<UserAssetStateTransition> {

        match self {
            UserAssetState::AssetServerRequestInFlight(state) => {
                return state.process(http_client);
            }
            UserAssetState::ClientLoadAssetRequestInFlight(state) => {
                return state.process(server);
            }
        }
    }
}

pub struct UserAssets {
    user_key: UserKey,
    assets_processing: HashMap<AssetId, UserAssetState>,
    assets_in_memory: HashSet<AssetId>,
}

impl UserAssets {
    pub fn new(user_key: &UserKey) -> Self {
        Self {
            user_key: *user_key,
            assets_processing: HashMap::new(),
            assets_in_memory: HashSet::new(),
        }
    }

    pub fn process_in_flight_requests(&mut self, server: &mut Server, http_client: &mut HttpClient, asset_store: &mut AssetStore) {

        let mut assets_finished = Vec::new();
        for (asset_id, user_asset_state) in self.assets_processing.iter_mut() {
            if let Some(transition) = user_asset_state.process_in_flight_requests(server, http_client) {
                match transition {
                    UserAssetStateTransition::AssetServerResponse(asset_etag, data_opt) => {
                        // received response from asset server
                        if let Some((dependencies, new_data)) = data_opt {
                            asset_store.insert_data(*asset_id, asset_etag, dependencies, new_data);
                        }

                        // send client "load asset" request
                        info!("sending load_asset request to client: (asset: {:?}, etag: {:?})", asset_id, asset_etag);
                        let client_load_asset_request = LoadAssetRequest::new(asset_id, &asset_etag);
                        let client_load_asset_response_key = server
                            .send_request::<RequestChannel, _>(&self.user_key, &client_load_asset_request)
                            .unwrap();

                        // move to next state
                        *user_asset_state = UserAssetState::ClientLoadAssetRequestInFlight(ClientLoadAssetRequestState::new(
                            client_load_asset_response_key,
                        ));
                    }
                    UserAssetStateTransition::ClientLoadAssetResponse(response_value) => {
                        match response_value {
                            LoadAssetResponseValue::ClientHasOldOrNoAsset => {
                                info!("sending asset data to client: {:?}", asset_id);

                                // get asset etag & data from store
                                let (etag, data) = asset_store.get_etag_and_data(asset_id).unwrap();
                                let message = LoadAssetWithData::new(*asset_id, etag, data);
                                server.send_message::<PrimaryChannel, _>(&self.user_key, &message);

                                // remove from processing, add to memory
                                assets_finished.push(*asset_id);
                            }
                            LoadAssetResponseValue::ClientLoadedNonModifiedAsset => {
                                // remove from processing, add to memory
                                assets_finished.push(*asset_id);
                            }
                        }
                    }
                }
            }
        }
        for asset_finished in assets_finished {
            self.assets_processing.remove(&asset_finished);
            self.assets_in_memory.insert(asset_finished);
        }
    }

    pub fn handle_user_asset_request(
        &mut self,
        http_client: &mut HttpClient,
        asset_server_addr: &str,
        asset_server_port: u16,
        asset_store: &AssetStore,
        asset_id: &AssetId,
        added: bool,
    ) {
        if added {
            self.handle_user_asset_added(
                http_client,
                asset_server_addr,
                asset_server_port,
                asset_store,
                asset_id,
            );
        } else {
            self.handle_user_asset_removed(asset_id);
        }
    }

    fn handle_user_asset_added(
        &mut self,
        http_client: &mut HttpClient,
        asset_server_addr: &str,
        asset_server_port: u16,
        asset_store: &AssetStore,
        asset_id: &AssetId,
    ) {
        // does user already have the asset in memory? if so, return
        // TODO: is there any reason we should check if the user has the right version in memory?
        if self.assets_in_memory.contains(asset_id) {
            return;
        }

        if self.assets_processing.contains_key(asset_id) {
            // this isn't necessarily an error, I just want to know if it's possible
            // world server shouldn't be sending the same request twice?
            panic!("user already has asset in processing: {:?}", asset_id);
            return;
        }

        // // check whether asset has any dependencies
        // let mut pending_dependencies = HashSet::new();
        // if let Some(dependencies) = asset_store.get_dependencies(asset_id) {
        //     for dependency in dependencies {
        //         if self.assets_in_memory.contains(dependency) {
        //             continue;
        //         } else {
        //
        //         }
        //     }
        // }

        // send 'asset' request to asset server
        info!("sending asset request to asset server: {:?}", asset_id);
        let etag_opt = asset_store.get_etag(asset_id);
        let asset_server_request = AssetRequest::new(*asset_id, etag_opt);
        let asset_server_response_key = http_client.send(
            asset_server_addr,
            asset_server_port,
            asset_server_request.clone(),
        );

        // save responsekeys for 'asset' request
        self.assets_processing.insert(*asset_id, UserAssetState::AssetServerRequestInFlight(AssetServerRequestState::new(
            asset_server_request,
            asset_server_response_key,
        )));
    }

    fn handle_user_asset_removed(&mut self, _asset_id: &AssetId) {
        todo!()
    }
}
