
mod animation;
pub use animation::{AnimFile, AnimFileFrame, AnimFileQuat};

mod icon;
pub use icon::{IconFile, IconFileFrame};

mod mesh;
pub use mesh::MeshFile;

mod model;
pub use model::ModelFile;

mod palette;
pub use palette::PaletteFile;

mod scene;
pub use scene::{SceneFile, SceneFileComponentType};

mod skeleton;
pub use skeleton::SkelFile;

mod skin;
pub use skin::SkinFile;

mod asset;
pub use asset::{Asset, AssetMeta, AssetData, AssetId};

pub const MAX_QUAT_COMPONENT_SIZE: f32 = 32.0;
pub const MAX_SCALE: f32 = 100.0;