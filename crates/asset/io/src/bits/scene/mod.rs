use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature = "read_bits")] {
        mod read;
        pub use read::*;
    } else {}
}

cfg_if! {
    if #[cfg(feature = "write_bits")] {
        mod write;
        pub use write::*;
    } else {}
}

use naia_serde::SerdeInternal as Serde;

use crate::AssetId;
use crate::bits::common::{FileTransformEntityType, SerdeQuat};

#[derive(Clone)]
pub enum SceneAction {
    // path, file_name, file_type
    Component(AssetId, FileTransformEntityType),
    // file index, x, y, z, scale_x, scale_y, scale_z, rotation
    NetTransform(u16, i16, i16, i16, f32, f32, f32, SerdeQuat),
}

#[derive(Serde, Clone, PartialEq)]
pub enum SceneActionType {
    SkinOrSceneFile,
    NetTransform,
    None,
}
