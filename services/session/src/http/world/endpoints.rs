use bevy_ecs::{system::Res, change_detection::ResMut};

use naia_bevy_server::Server;

use logging::{info, warn};
use bevy_http_client::{HttpClient, ResponseError};
use bevy_http_server::HttpServer;

use session_server_http_proto::{UserAssetIdRequest, UserAssetIdResponse};

use crate::{http::world::WorldConnections, asset::asset_manager::AssetManager};

pub fn recv_added_asset_id_request(
    world_connections: Res<WorldConnections>,
    mut http_server: ResMut<HttpServer>,
    mut naia_server: Server,
    mut http_client: ResMut<HttpClient>,
    mut asset_manager: ResMut<AssetManager>,
) {
    while let Some((_addr, request, response_key)) = http_server.receive::<UserAssetIdRequest>() {
        let world_instance_secret = request.world_instance_secret();

        if !world_connections.world_instance_exists(world_instance_secret) {
            warn!("invalid request secret");
            http_server.respond(response_key, Err(ResponseError::Unauthenticated));
            continue;
        }

        let user_id = request.user_id();
        let asset_id = request.asset_id();
        let added = request.added();

        info!(
            "received from worldserver: user_asset_request(user_id: {:?}, asset_id: {:?})",
            user_id, asset_id
        );

        let user_key = world_connections
            .get_user_key_from_world_instance(world_instance_secret, &user_id)
            .unwrap();

        if added {
            let asset_server_url = asset_manager.get_asset_server_url();
            asset_manager.load_user_asset(
                &mut naia_server,
                &mut http_client,
                &asset_server_url,
                user_key,
                asset_id,
            );
        } else {
            let asset_server_url = asset_manager.get_asset_server_url();
            asset_manager.unload_user_asset(&asset_server_url, user_key, asset_id);
        }

        //info!("UserAsset Response sent to world server ..");

        http_server.respond(response_key, Ok(UserAssetIdResponse));
    }

    if asset_manager.has_queued_user_asset_requests() {
        if let Some((asset_server_addr, asset_server_port)) = asset_manager.get_asset_server_url() {
            asset_manager.process_queued_user_asset_requests(
                &mut naia_server,
                &mut http_client,
                &asset_server_addr,
                asset_server_port,
            );
        }
    }
}
