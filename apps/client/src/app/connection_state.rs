use game_engine::{http::ResponseKey, orchestrator::LoginResponse};

#[derive(Clone, PartialEq)]
pub enum ConnectionState {
    Disconnected,
    SentToOrchestrator(ResponseKey<LoginResponse>),
    ReceivedFromOrchestrator(LoginResponse),
    ConnectedToSession,
    ConnectedToWorld,
}