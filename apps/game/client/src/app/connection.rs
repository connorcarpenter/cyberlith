use game_engine::orchestrator::OrchestratorRequestKey;

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum ConnectionState {
    Disconnected,
    SentToOrchestrator(OrchestratorRequestKey),
    Connected,
}