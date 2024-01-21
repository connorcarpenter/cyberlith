use game_engine::{bevy_http_client::ResponseKey, orchestrator::LoginResponse};

#[derive(Clone, PartialEq)]
pub enum ConnectionState {
    Disconnected,
    SentToOrchestrator(ResponseKey<LoginResponse>),
    Connected,
}
