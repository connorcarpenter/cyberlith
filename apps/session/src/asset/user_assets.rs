
use std::collections::{HashMap, HashSet};

use bevy_log::info;

use naia_bevy_server::{Server, UserKey};

use session_server_naia_proto::{channels::PrimaryChannel, messages::{LoadAssetWithData, LoadAssetResponseValue}};

use asset_id::AssetId;
use bevy_http_client::HttpClient;

use crate::asset::{asset_store::AssetStore, user_asset_processing::{UserAssetProcessingState, UserAssetProcessingStateTransition}};

pub struct UserAssets {
    user_key: UserKey,
    assets_processing: HashMap<AssetId, UserAssetProcessingState>,
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

        let mut state_transitions = Vec::new();
        for (asset_id, user_asset_state) in self.assets_processing.iter_mut() {
            if let Some(transition) = user_asset_state.process(server, http_client) {
                state_transitions.push((*asset_id, transition));
            }
        }

        for (asset_id, transition) in state_transitions {
            match transition {
                UserAssetProcessingStateTransition::AssetServerResponse(asset_etag, data_opt) => {
                    // received response from asset server
                    if let Some((dependencies, new_data)) = data_opt {
                        asset_store.insert_data(asset_id, asset_etag, dependencies, new_data);
                    }

                    // move to next state
                    self.assets_processing.insert(asset_id, UserAssetProcessingState::send_client_load_asset_request(
                        server,
                        &self.user_key,
                        &asset_id,
                        &asset_etag,
                    ));
                }
                UserAssetProcessingStateTransition::ClientLoadAssetResponse(response_value) => {
                    match response_value {
                        LoadAssetResponseValue::ClientHasOldOrNoAsset => {
                            info!("sending asset data to client: {:?}", asset_id);

                            // get asset etag & data from store
                            let (asset_etag, asset_data) = asset_store.get_etag_and_data(&asset_id).unwrap();

                            // send asset data to client
                            let message = LoadAssetWithData::new(asset_id, asset_etag, asset_data);
                            server.send_message::<PrimaryChannel, _>(&self.user_key, &message);

                            // remove from processing, add to memory
                            self.assets_processing.remove(&asset_id);
                            self.assets_in_memory.insert(asset_id);
                        }
                        LoadAssetResponseValue::ClientLoadedNonModifiedAsset => {
                            // remove from processing, add to memory
                            self.assets_processing.remove(&asset_id);
                            self.assets_in_memory.insert(asset_id);
                        }
                    }
                }
            }
        }
    }

    pub fn handle_user_asset_request(
        &mut self,
        server: &mut Server,
        http_client: &mut HttpClient,
        asset_server_addr: &str,
        asset_server_port: u16,
        asset_store: &AssetStore,
        asset_id: &AssetId,
        added: bool,
    ) {
        if added {
            self.handle_user_asset_added(
                server,
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
        server: &mut Server,
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
            //return;
        }

        // check whether asset has any dependencies
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

        if asset_store.has_asset(asset_id) {
            // asset is in session_server's store, but not in client's memory
            let asset_etag = asset_store.get_etag(asset_id).unwrap();

            // move to next state
            self.assets_processing.insert(
                *asset_id,
                UserAssetProcessingState::send_client_load_asset_request(
                    server,
                    &self.user_key,
                    asset_id,
                    &asset_etag,
                )
            );
        } else {
            // asset is not in store
            // send 'asset' request to asset server
            self.assets_processing.insert(
                *asset_id,
                UserAssetProcessingState::send_asset_server_request(
                    http_client,
                    asset_server_addr,
                    asset_server_port,
                    asset_id,
                    None,
                )
            );
        }
    }

    fn handle_user_asset_removed(&mut self, _asset_id: &AssetId) {
        todo!()
    }
}
