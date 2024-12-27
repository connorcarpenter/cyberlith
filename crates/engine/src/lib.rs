#[macro_use]
extern crate cfg_if;

mod plugin;
pub use plugin::EnginePlugin;

mod renderer;

pub mod kernel {
    pub use kernel::{executor, get_querystring_param, AppExitAction, KernelApp};
}
pub mod asset {
    pub use asset_cache::AssetLoadedEvent;
    pub use asset_id::{AssetId, AssetType, ETag};
    pub use asset_loader::{
        embedded_asset_event, AnimatedModelData, AnimationData, AssetHandle, AssetManager,
        AssetMetadataSerde, EmbeddedAssetEvent, IconData, MeshData, ModelData, PaletteData,
        SceneData, SkeletonData, SkinData, UnitData,
    };
    pub use asset_render::AssetRender;
}
pub mod time {
    pub use instant::Instant;
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
    pub use ui_runner::{state::NodeActiveState, UiHandle, UiManager};

    pub mod extensions {
        pub use ui_extensions::*;
    }
}
pub mod random {
    pub use random::*;
}
pub mod http {
    pub use kernel::http::*;
}
pub use logging;
// pub mod auth {
//     pub use auth_server_types::*;
// }
pub mod social {
    pub use social_server_types::*;
}
