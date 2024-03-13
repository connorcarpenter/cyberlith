#[macro_use]
extern crate cfg_if;

mod plugin;
pub use plugin::{EnginePlugin, NetworkedEnginePlugin};

mod asset_cache;
mod asset_ref_processor;
mod client_markers;
mod connection_manager;
mod embedded_asset;
mod renderer;
mod world_events;

pub mod asset {
    pub use asset_id::{AssetId, AssetType};
    pub use asset_render::{
        embedded_asset_event, AnimationData, AssetHandle, AssetManager, EmbeddedAssetEvent,
        IconData, MeshData, ModelData, PaletteData, SceneData, SkeletonData, SkinData, TextStyle,
    };
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
    pub use super::world_events::InsertAssetRefEvent as WorldInsertAssetRefEvent;
    pub use super::world_events::InsertComponentEvent as WorldInsertComponentEvent;
    pub use world_server_naia_proto::components::{Alt1, Main, Position};
}
pub mod config {
    pub use config::*;
}
pub mod storage {
    pub use storage::*;
}
pub mod ui {
    pub use ui::{Alignment, NodeId, UiPlugin, Ui};
}

// TODO: should these find a home?
pub use renderer::wait_for_finish;

pub use world_events::InsertComponentEvent;
