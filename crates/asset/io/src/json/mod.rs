use cfg_if::cfg_if;

mod animation;
pub use animation::{AnimationJson, AnimFileFrame, AnimFileQuat};

mod icon;
pub use icon::{IconJson, IconFileFrame};

mod mesh;
pub use mesh::MeshJson;

mod model;
pub use model::ModelJson;

mod palette;
pub use palette::PaletteJson;

mod scene;
pub use scene::SceneJson;

mod skeleton;
pub use skeleton::SkeletonJson;

mod skin;
pub use skin::SkinJson;

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
