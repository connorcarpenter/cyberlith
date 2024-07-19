mod asset_cache_checker;
mod asset_ref_processor;
mod client_markers;
mod connection_manager;
mod networked_plugin;
mod session_events;
mod world_events;

pub mod world {
    use naia_bevy_client::{events::{RejectEvent, DisconnectEvent, SpawnEntityEvent, ConnectEvent, DespawnEntityEvent, ErrorEvent}, Client};

    use super::client_markers::World;

    pub type WorldClient<'w> = Client<'w, World>;
    pub type WorldConnectEvent = ConnectEvent<World>;
    pub type WorldDisconnectEvent = DisconnectEvent<World>;
    pub type WorldRejectEvent = RejectEvent<World>;
    pub type WorldSpawnEntityEvent = SpawnEntityEvent<World>;
    pub type WorldDespawnEntityEvent = DespawnEntityEvent<World>;
    pub type WorldErrorEvent = ErrorEvent<World>;

    pub use super::world_events::{
        InsertAssetRefEvent as WorldInsertAssetRefEvent, WorldInsertComponentEvent,
        WorldRemoveComponentEvent, WorldUpdateComponentEvent,
    };

    pub use world_server_naia_proto::{channels, components, messages};
}

pub mod session {

    use naia_bevy_client::{events::{RejectEvent, ConnectEvent, DisconnectEvent, SpawnEntityEvent, DespawnEntityEvent, ErrorEvent}, Client};

    use super::client_markers::Session;

    pub type SessionClient<'w> = Client<'w, Session>;
    pub type SessionConnectEvent = ConnectEvent<Session>;
    pub type SessionDisconnectEvent = DisconnectEvent<Session>;
    pub type SessionRejectEvent = RejectEvent<Session>;
    pub type SessionSpawnEntityEvent = SpawnEntityEvent<Session>;
    pub type SessionDespawnEntityEvent = DespawnEntityEvent<Session>;
    pub type SessionErrorEvent = ErrorEvent<Session>;

    pub use super::session_events::{
        SessionInsertComponentEvent, SessionRemoveComponentEvent, SessionUpdateComponentEvent,
    };

    pub use session_server_naia_proto::{channels, components, messages};
}

pub use networked_plugin::NetworkedEnginePlugin;
