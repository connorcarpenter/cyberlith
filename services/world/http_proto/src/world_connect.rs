
use naia_serde::SerdeInternal as Serde;

use auth_server_types::UserId;
use bevy_http_shared::{ApiRequest, ApiResponse, Method};
use social_server_types::LobbyId;

// this is sent by the region server

// Request
#[derive(Serde, PartialEq, Clone)]
pub struct WorldConnectRequest {
    region_secret: String,
    lobby_id: LobbyId,
    // session server addr, session server port, Vec<(UserId, login_token)>
    login_tokens: Vec<(String, u16, Vec<(UserId, String)>)>,
}

impl WorldConnectRequest {
    pub fn new(
        region_secret: &str,
        lobby_id: LobbyId,
        login_tokens: Vec<(String, u16, Vec<(UserId, String)>)>,
    ) -> Self {
        Self {
            lobby_id,
            region_secret: region_secret.to_string(),
            login_tokens,
        }
    }

    pub fn region_secret(&self) -> &str {
        &self.region_secret
    }

    pub fn lobby_id(&self) -> LobbyId {
        self.lobby_id
    }

    pub fn login_tokens(&self) -> &Vec<(String, u16, Vec<(UserId, String)>)> {
        &self.login_tokens
    }
}

// Response
#[derive(Serde, PartialEq, Clone)]
pub struct WorldConnectResponse;

impl WorldConnectResponse {
    pub fn new() -> Self {
        Self
    }
}

// Traits
impl ApiRequest for WorldConnectRequest {
    type Response = WorldConnectResponse;

    fn name() -> &'static str {
        "WorldConnectRequest"
    }

    fn method() -> Method {
        Method::Post
    }

    fn path() -> &'static str {
        "world/connect"
    }
}

impl ApiResponse for WorldConnectResponse {
    fn name() -> &'static str {
        "WorldConnectResponse"
    }
}
