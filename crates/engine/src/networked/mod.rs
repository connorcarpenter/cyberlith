mod asset_cache_checker;
mod asset_ref_processor;
mod client_markers;
mod connection_manager;
mod component_events;
mod networked_plugin;
mod session_events;
mod world_events;

pub mod world {
    use naia_bevy_client::{events::SpawnEntityEvent, Client};

    use super::client_markers::World;

    pub type WorldClient<'w> = Client<'w, World>;
    pub type WorldSpawnEntityEvent = SpawnEntityEvent<World>;
    pub type WorldDespawnEntityEvent = naia_bevy_client::events::DespawnEntityEvent<World>;
    pub type WorldErrorEvent = naia_bevy_client::events::ErrorEvent<World>;

    pub use super::world_events::{
        InsertAssetRefEvent as WorldInsertAssetRefEvent, WorldInsertComponentEvent, WorldUpdateComponentEvent, WorldRemoveComponentEvent
    };

    pub use world_server_naia_proto::{channels, components, messages};
}

pub mod session {

    use naia_bevy_client::{events::SpawnEntityEvent, Client};

    use super::client_markers::Session;

    pub type SessionClient<'w> = Client<'w, Session>;
    pub type SessionSpawnEntityEvent = SpawnEntityEvent<Session>;
    pub type SessionDespawnEntityEvent = naia_bevy_client::events::DespawnEntityEvent<Session>;
    pub type SessionErrorEvent = naia_bevy_client::events::ErrorEvent<Session>;

    pub use super::session_events::{SessionInsertComponentEvent, SessionUpdateComponentEvent, SessionRemoveComponentEvent};

    pub use session_server_naia_proto::{channels, components, messages};
}

pub use networked_plugin::NetworkedEnginePlugin;
