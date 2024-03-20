#[macro_use]
extern crate cfg_if;

mod plugin;
pub use plugin::EnginePlugin;

cfg_if! {
    if #[cfg(feature = "networked")] {
        mod networked;
        pub use networked::*;
    } else {}
}

mod asset_cache;
mod embedded_asset;
mod renderer;

pub mod asset {
    pub use asset_id::{AssetId, AssetType, ETag};
    pub use asset_render::{
        AnimationData, AssetHandle, AssetManager, embedded_asset_event, EmbeddedAssetEvent,
        IconData, MeshData, ModelData, PaletteData, SceneData, SkeletonData, SkinData, UiData, AssetMetadataSerde,
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
pub mod config {
    pub use config::*;
}
pub mod storage {
    pub use storage::*;
}
pub mod ui {
    pub use ui::{Alignment, NodeId, Ui};
}

// TODO: should these find a home?
pub use renderer::wait_for_finish;
