mod asset_ref_processor;
mod asset_cache_checker;
mod networked_plugin;
mod connection_manager;
mod client_markers;
mod world_events;

pub mod world {
    use naia_bevy_client::{Client, events::SpawnEntityEvent};

    use super::client_markers::World;

    pub type WorldClient<'w> = Client<'w, World>;
    pub type WorldSpawnEntityEvent = SpawnEntityEvent<World>;
    pub use super::world_events::InsertAssetRefEvent as WorldInsertAssetRefEvent;
    pub use super::world_events::InsertComponentEvent as WorldInsertComponentEvent;

    pub use world_server_naia_proto::components::{Alt1, Main, Position};
}

pub use networked_plugin::NetworkedEnginePlugin;