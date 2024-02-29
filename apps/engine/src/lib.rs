#[macro_use]
extern crate cfg_if;

mod plugin;
pub use plugin::EnginePlugin;

mod client_markers;
mod renderer;
mod asset_cache;
mod connection_manager;
mod world_events;
mod asset_ref_processor;

pub mod asset {
    pub use asset_render::{AssetManager, AnimationData, AssetHandle, IconData, MeshData, ModelData, PaletteData,SceneData,SkeletonData,SkinData, TextStyle};
    pub use asset_id::{AssetId, AssetType};
}
pub mod input {
    pub use input::*;
}
pub mod math {
    pub use math::*;
}
pub mod render {
    pub use render_api::*;
}
pub mod world {
    use naia_bevy_client::{events::SpawnEntityEvent, Client};

    use super::client_markers::World;

    pub type WorldClient<'w> = Client<'w, World>;
    pub type WorldSpawnEntityEvent = SpawnEntityEvent<World>;
    pub use super::world_events::InsertComponentEvent as WorldInsertComponentEvent;
    pub use super::world_events::InsertAssetRefEvent as WorldInsertAssetRefEvent;
    pub use world_server_naia_proto::components::{Position, Main, Alt1};
}
pub mod config {
    pub use config::*;
}
pub mod storage {
    pub use storage::*;
}

// TODO: should these find a home?
pub use renderer::wait_for_finish;

pub use world_events::InsertComponentEvent;