mod animation;
mod asset_dependency;
mod asset_handle;
mod asset_manager;
mod icon;
mod mesh;
mod model;
mod palette;
mod plugin;
mod scene;
mod skeleton;
mod skin;
mod asset_renderer;
mod asset_store;

pub use animation::*;
pub use asset_handle::*;
pub use asset_manager::*;
pub use icon::*;
pub use mesh::*;
pub use model::*;
pub use palette::*;
pub use plugin::*;
pub use scene::*;
pub use skeleton::*;
pub use skin::*;

use asset_id::AssetId;
pub(crate) fn data_from_asset_id(asset_id: &AssetId) -> Result<Vec<u8>, std::io::Error> {
    todo!()
}