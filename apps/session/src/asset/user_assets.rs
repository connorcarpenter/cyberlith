
use std::collections::{HashMap, HashSet};

use bevy_log::info;

use naia_bevy_server::{Server, UserKey};

use session_server_naia_proto::{channels::PrimaryChannel, messages::{LoadAssetWithData, LoadAssetResponseValue}};

use asset_id::{AssetId, ETag};
use bevy_http_client::HttpClient;

use crate::asset::{asset_store::AssetStore, user_asset_processing::{UserAssetProcessingState, UserAssetProcessingStateTransition}};

pub struct UserAssets {
    user_key: UserKey,
    assets_processing: HashMap<AssetId, UserAssetProcessingState>,
    assets_in_memory: HashSet<AssetId>,
    // when KEY is loaded in client, notify all VALUES that they can now load
    dependency_waitlist: HashMap<AssetId, HashSet<AssetId>>,
}

impl UserAssets {
    pub fn new(user_key: &UserKey) -> Self {
        Self {
            user_key: *user_key,
            assets_processing: HashMap::new(),
            assets_in_memory: HashSet::new(),
            dependency_waitlist: HashMap::new(),
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

        if asset_store.has_asset(asset_id) {
            // asset is in session_server's store, but not in client's memory
            let asset_etag = asset_store.get_etag(asset_id).unwrap();

            // move to next state
            self.handle_asset_dependencies(server, http_client, asset_server_addr, asset_server_port, asset_store, asset_id, &asset_etag);
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

    pub fn process_in_flight_requests(
        &mut self,
        server: &mut Server,
        http_client: &mut HttpClient,
        asset_server_addr: &str,
        asset_server_port: u16,
        asset_store: &mut AssetStore
    ) {

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
                    if let Some((asset_type, dependencies, new_data)) = data_opt {
                        asset_store.insert_data(asset_id, asset_type, asset_etag, dependencies, new_data);
                    }

                    // move to next state
                    self.handle_asset_dependencies(server, http_client, asset_server_addr, asset_server_port, asset_store, &asset_id, &asset_etag);
                }
                UserAssetProcessingStateTransition::ClientLoadAssetResponse(response_value) => {
                    match response_value {
                        LoadAssetResponseValue::ClientHasOldOrNoAsset => {
                            info!("sending asset data to client: {:?}", asset_id);

                            // get asset etag & data from store
                            let (asset_type, asset_etag, asset_data) = asset_store.get_type_and_etag_and_data(&asset_id).unwrap();

                            // send asset data to client
                            let message = LoadAssetWithData::new(asset_id, asset_type, asset_etag, asset_data);
                            server.send_message::<PrimaryChannel, _>(&self.user_key, &message);

                            // remove from processing, add to memory
                            self.finish_asset_processing(server, asset_store, &asset_id);
                        }
                        LoadAssetResponseValue::ClientLoadedNonModifiedAsset => {
                            // remove from processing, add to memory
                            self.finish_asset_processing(server, asset_store, &asset_id);
                        }
                    }
                }
            }
        }
    }

    fn finish_asset_processing(
        &mut self,
        server: &mut Server,
        asset_store: &AssetStore,
        asset_id: &AssetId
    ) {
        info!("finished processing asset: {:?}", asset_id);

        self.assets_processing.remove(&asset_id);
        self.assets_in_memory.insert(*asset_id);

        // handle dependency waitlist
        if let Some(waiting_asset_ids) = self.dependency_waitlist.remove(asset_id) {
            for waiting_asset_id in waiting_asset_ids {

                info!("asset: {:?} has finished loading, notifying waiting asset: {:?}", asset_id, waiting_asset_id);

                let waiting_asset_state = self.assets_processing.get_mut(&waiting_asset_id).unwrap();
                let waiting_asset_finished = waiting_asset_state.handle_dependency_loaded(asset_id);

                // if waiting asset has no more dependencies, move it to next state
                if waiting_asset_finished {

                    info!("all dependencies for asset: {:?} have been loaded, sending to client", waiting_asset_id);

                    // move to next state
                    let waiting_asset_etag = asset_store.get_etag(&waiting_asset_id).unwrap();
                    // all of asset's dependencies are already loaded (or asset has no dependencies), send asset over to client
                    self.assets_processing.insert(waiting_asset_id, UserAssetProcessingState::send_client_load_asset_request(
                        server,
                        &self.user_key,
                        &waiting_asset_id,
                        &waiting_asset_etag,
                    ));
                }
            }
        }
    }

    pub fn handle_asset_dependencies(
        &mut self,
        server: &mut Server,
        http_client: &mut HttpClient,
        asset_server_addr: &str,
        asset_server_port: u16,
        asset_store: &AssetStore,
        asset_id: &AssetId,
        asset_etag: &ETag
    ) {
        // check whether asset has any dependencies
        let mut pending_dependencies = HashSet::new();
        if let Some(dependencies) = asset_store.get_dependencies(asset_id) {
            for dependency in dependencies {
                if self.assets_in_memory.contains(dependency) {
                    // dependency is already loaded in client's memory
                    continue;
                }

                if !self.assets_processing.contains_key(dependency) {
                    // add task to process dependency if it doesn't exist
                    self.handle_user_asset_added(server, http_client, asset_server_addr, asset_server_port, asset_store, dependency);
                }

                // add dependency to pending list
                pending_dependencies.insert(*dependency);

                // add asset to dependency waitlist
                if !self.dependency_waitlist.contains_key(dependency) {
                    self.dependency_waitlist.insert(*dependency, HashSet::new());
                }
                let dependency_alertlist = self.dependency_waitlist.get_mut(dependency).unwrap();
                dependency_alertlist.insert(*asset_id);
            }
        }

        if pending_dependencies.is_empty() {
            // all of asset's dependencies are already loaded (or asset has no dependencies), send asset over to client
            self.assets_processing.insert(*asset_id, UserAssetProcessingState::send_client_load_asset_request(
                server,
                &self.user_key,
                asset_id,
                asset_etag,
            ));
        } else {
            // asset must wait on dependencies to load first
            self.assets_processing.insert(*asset_id, UserAssetProcessingState::waiting_for_dependencies(
                pending_dependencies,
            ));
        }
    }
}
