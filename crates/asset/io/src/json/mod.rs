use cfg_if::cfg_if;

mod animation;
pub use animation::{AnimFile, AnimFileFrame, AnimFileQuat};

mod icon;
pub use icon::{IconFile, IconFileFrame};

mod mesh;
pub use mesh::MeshData;

mod model;
pub use model::ModelFile;

mod palette;
pub use palette::PaletteFile;

mod scene;
pub use scene::SceneFile;

mod skeleton;
pub use skeleton::SkelFile;

mod skin;
pub use skin::SkinFile;

mod ui;
pub use ui::*;

mod asset;
pub use asset::{Asset, AssetData, AssetMeta};

cfg_if! {
    if #[cfg(feature = "read_json")] {
        pub use asset::ProcessedAssetMeta;
    } else {}
}

mod components;

pub use components::FileComponentType;

pub const MAX_QUAT_COMPONENT_SIZE: f32 = 32.0;
pub const MAX_SCALE: f32 = 100.0;
