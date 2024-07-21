pub mod asset_manager;
pub mod asset_store;

mod user_asset_processing;
mod user_assets;

cfg_if::cfg_if!(
    if #[cfg(feature = "odst")] {} else {
        mod ui_asset_catalog;
        pub use ui_asset_catalog::*;
    }
);

mod plugin;
pub use plugin::*;
