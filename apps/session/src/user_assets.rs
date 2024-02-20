use bevy_log::info;
use std::collections::HashSet;

use naia_bevy_server::{ResponseReceiveKey, Server, UserKey};

use asset_id::{AssetId, ETag};
use bevy_http_client::{HttpClient, ResponseKey};

use asset_server_http_proto::{AssetRequest, AssetResponse, AssetResponseValue};
use session_server_naia_proto::channels::PrimaryChannel;
use session_server_naia_proto::messages::{AssetDataMessage, AssetEtagResponseValue};
use session_server_naia_proto::{
    channels::RequestChannel,
    messages::{AssetEtagRequest, AssetEtagResponse},
};

use crate::asset_cache::AssetCache;

struct FirstFlightRequest {
    client_etag_response_key: Option<ResponseReceiveKey<AssetEtagResponse>>,
    asset_server_response_key: Option<ResponseKey<AssetResponse>>,
    client_etag_response: Option<AssetEtagResponse>,
    asset_server_response: Option<AssetResponse>,
    asset_server_request: AssetRequest,
}

impl FirstFlightRequest {
    pub fn new(
        client_etag_response_key: ResponseReceiveKey<AssetEtagResponse>,
        asset_server_request: AssetRequest,
        asset_server_response_key: ResponseKey<AssetResponse>,
    ) -> Self {
        Self {
            client_etag_response_key: Some(client_etag_response_key),
            asset_server_response_key: Some(asset_server_response_key),
            client_etag_response: None,
            asset_server_request,
            asset_server_response: None,
        }
    }

    pub fn process(&mut self, server: &mut Server, http_client: &mut HttpClient) {
        if let Some(key) = self.client_etag_response_key.as_ref() {
            if let Some((_user_key, response)) = server.receive_response(key) {
                info!("received asset etag response from client");
                self.client_etag_response_key = None;
                self.client_etag_response = Some(response);
            }
        }
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

    pub fn both_completed(&self) -> bool {
        self.client_etag_response.is_some() && self.asset_server_response.is_some()
    }

    pub fn unwrap_responses(self) -> (AssetEtagResponse, AssetRequest, AssetResponse) {
        (
            self.client_etag_response.unwrap(),
            self.asset_server_request,
            self.asset_server_response.unwrap(),
        )
    }
}

pub struct UserAssets {
    user_key: UserKey,
    assets_in_memory: HashSet<AssetId>,
    first_flight_requests: Vec<FirstFlightRequest>,
}

impl UserAssets {
    pub fn new(user_key: &UserKey) -> Self {
        Self {
            user_key: *user_key,
            assets_in_memory: HashSet::new(),
            first_flight_requests: Vec::new(),
        }
    }

    pub fn user_asset_request(
        &mut self,
        server: &mut Server,
        http_client: &mut HttpClient,
        asset_server_addr: &str,
        asset_server_port: u16,
        asset_store: &AssetCache,
        asset_id: &AssetId,
        added: bool,
    ) {
        if added {
            self.user_asset_added(
                server,
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

    pub fn process_in_flight_requests(
        &mut self,
        server: &mut Server,
        http_client: &mut HttpClient,
    ) -> (
        Option<Vec<(AssetId, ETag, Vec<u8>)>>,
        Option<Vec<(UserKey, AssetId)>>,
    ) {
        let first_flight_requests = std::mem::take(&mut self.first_flight_requests);

        if first_flight_requests.is_empty() {
            return (None, None);
        }

        let mut first_flight_responses = Vec::new();

        for mut request in first_flight_requests {
            request.process(server, http_client);
            if request.both_completed() {
                first_flight_responses.push(request.unwrap_responses());
            } else {
                self.first_flight_requests.push(request);
            }
        }

        if first_flight_responses.is_empty() {
            return (None, None);
        }

        let mut asset_server_responses = Vec::new();
        let mut pending_client_requests = Vec::new();
        for (client_etag_res, asset_server_req, asset_res) in first_flight_responses {
            let asset_id = asset_server_req.asset_id();
            let old_etag_opt = asset_server_req.etag_opt();

            match (client_etag_res.value, asset_res.value) {
                (
                    AssetEtagResponseValue::Found(client_etag),
                    AssetResponseValue::Modified(new_etag, data),
                ) => {
                    if client_etag == new_etag {
                        panic!(
                            "client somehow has newer etag than server, this should never happen"
                        )
                    }

                    info!("client responded with old etag: {:?}, asset server responded with newer etag: {:?}", client_etag, new_etag);
                    info!(
                        "storing asset data for: {:?}, and sending new data to client",
                        asset_id
                    );

                    // store new asset etag & data
                    asset_server_responses.push((asset_id, new_etag, data));
                    // send asset etag & data to client
                    pending_client_requests.push((self.user_key, asset_id));
                }
                (AssetEtagResponseValue::Found(client_etag), AssetResponseValue::NotModified) => {
                    if let Some(old_etag) = old_etag_opt {
                        if old_etag == client_etag {
                            // client already has latest asset, done!

                            info!("client already has latest asset: {:?}", asset_id);

                            self.assets_in_memory.insert(asset_id);
                            continue;
                        } else {
                            // send asset etag & data to client

                            info!("client has old asset, sending new data: {:?}", asset_id);

                            pending_client_requests.push((self.user_key, asset_id));
                        }
                    } else {
                        panic!("asset server responded with NotModified but no etag was provided in request... this should never happen");
                    }
                }
                (
                    AssetEtagResponseValue::NotFound,
                    AssetResponseValue::Modified(new_etag, data),
                ) => {
                    info!(
                        "client responded with no etag, asset server responded with new etag: {:?}",
                        new_etag
                    );
                    info!(
                        "storing asset data for: {:?}, and sending new data to client",
                        asset_id
                    );

                    // store new asset etag & data
                    asset_server_responses.push((asset_id, new_etag, data));
                    // send asset etag & data to client
                    pending_client_requests.push((self.user_key, asset_id));
                }
                (AssetEtagResponseValue::NotFound, AssetResponseValue::NotModified) => {
                    if old_etag_opt.is_none() {
                        panic!("asset server responded with NotModified but no etag was provided in request... this should never happen");
                    }

                    info!("client responded with no etag, asset server responded with NotModified");
                    info!("sending asset data for: {:?} to client", asset_id);

                    // send asset etag & data to client
                    pending_client_requests.push((self.user_key, asset_id));
                }
            }
        }

        let asset_server_responses = if asset_server_responses.is_empty() {
            None
        } else {
            Some(asset_server_responses)
        };
        let pending_client_requests = if pending_client_requests.is_empty() {
            None
        } else {
            Some(pending_client_requests)
        };
        (asset_server_responses, pending_client_requests)
    }

    pub(crate) fn send_client_asset_data(
        &mut self,
        server: &mut Server,
        asset_store: &AssetCache,
        asset_id: &AssetId,
    ) {
        info!("sending asset data to client: {:?}", asset_id);

        // get asset etag & data from store
        let (etag, data) = asset_store.get_etag_and_data(asset_id).unwrap();
        let message = AssetDataMessage::new(*asset_id, etag, data);
        server.send_message::<PrimaryChannel, _>(&self.user_key, &message);

        // mark asset as in memory
        self.assets_in_memory.insert(*asset_id);
    }

    fn user_asset_added(
        &mut self,
        server: &mut Server,
        http_client: &mut HttpClient,
        asset_server_addr: &str,
        asset_server_port: u16,
        asset_store: &AssetCache,
        asset_id: &AssetId,
    ) {
        // does user already have the asset in memory? if so, return
        if self.assets_in_memory.contains(asset_id) {
            return;
        }

        // send 'asset_etag' request to client
        info!("sending asset etag request to client: {:?}", asset_id);
        let asset_etag_request = AssetEtagRequest::new(asset_id);
        let asset_etag_response_key = server
            .send_request::<RequestChannel, _>(&self.user_key, &asset_etag_request)
            .unwrap();

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
        self.first_flight_requests.push(FirstFlightRequest::new(
            asset_etag_response_key,
            asset_server_request,
            asset_server_response_key,
        ));
    }

    fn user_asset_removed(&mut self, _asset_id: &AssetId) {
        todo!()
    }
}
