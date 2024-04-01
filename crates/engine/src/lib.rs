#[macro_use]
extern crate cfg_if;

cfg_if! {
    if #[cfg(feature = "networked")] {
        mod networked;
        pub use networked::*;
    } else {}
}

mod plugin;
pub use plugin::EnginePlugin;

mod asset_cache;
mod embedded_asset;
mod renderer;

pub mod asset {
    pub use asset_id::{AssetId, AssetType, ETag};
    pub use asset_loader::{
        embedded_asset_event, AnimationData, AssetHandle, AssetManager, AssetMetadataSerde,
        EmbeddedAssetEvent, IconData, MeshData, ModelData, PaletteData, SceneData, SkeletonData,
        SkinData,
    };
    pub use asset_render::AssetRender;
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
    pub use ui_runtime::{UiManager, UiRuntime};
    pub use ui_render::UiRender;
    pub use ui_input::UiInputConverter;
}
pub mod random {
    pub use random::*;
}

// TODO: should these find a home?
pub use renderer::wait_for_finish;
