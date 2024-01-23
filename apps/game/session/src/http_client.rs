
use bevy_ecs::change_detection::ResMut;
use bevy_log::{info, warn};

use naia_bevy_server::Server;
use bevy_http_client::HttpClient;

use session_server_naia_proto::{channels::PrimaryChannel, messages::WorldConnectToken};

use crate::global::Global;

pub fn world_connect_recv(
    mut server: Server,
    mut http_client: ResMut<HttpClient>,
    mut global: ResMut<Global>,
) {
    for (response_key, user_key) in global.world_keys() {
        if let Some(result) = http_client.recv(response_key) {
            match result {
                Ok(response) => {
                    info!("received from regionserver: (addr: {:?}, token: {:?})", response.world_server_addr, response.token);

                    let token = WorldConnectToken::new(
                        response.world_server_addr.inner(),
                        &response.token,
                    );
                    server.send_message::<PrimaryChannel, WorldConnectToken>(user_key, &token);
                }
                Err(_) => {
                    warn!("error receiving message from region server..");
                }
            }
        }
    }
}