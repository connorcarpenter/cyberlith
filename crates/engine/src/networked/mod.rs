mod asset_cache_checker;
mod asset_ref_processor;
mod client_markers;
mod connection_manager;
mod networked_plugin;
mod world_events;

pub mod world {
    use naia_bevy_client::{events::SpawnEntityEvent, Client};

    use super::client_markers::World;

    pub type WorldClient<'w> = Client<'w, World>;
    pub type WorldSpawnEntityEvent = SpawnEntityEvent<World>;
    pub use super::world_events::{InsertComponentEvent as WorldInsertComponentEvent, InsertAssetRefEvent as WorldInsertAssetRefEvent};

    pub use world_server_naia_proto::components::{Alt1, Main, Position};
}

pub mod session {
    use naia_bevy_client::Client;

    use super::client_markers::Session;

    pub type SessionClient<'w> = Client<'w, Session>;

    pub use session_server_naia_proto::{channels::ClientActionsChannel, messages::WorldConnectRequest};
}

pub use networked_plugin::NetworkedEnginePlugin;
