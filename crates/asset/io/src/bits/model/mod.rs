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

use crate::bits::common::{FileTransformEntityType, SerdeQuat};

// Actions
#[derive(Clone)]
pub enum ModelAction {
    // path, file_name
    SkelFile(String, String),
    // path, file_name, file_type
    SkinOrSceneFile(String, String, FileTransformEntityType),
    // file index, name, x, y, z, scale_x, scale_y, scale_z, rotation
    NetTransform(u16, String, i16, i16, i16, f32, f32, f32, SerdeQuat),
}

#[derive(Serde, Clone, PartialEq)]
pub enum ModelActionType {
    SkelFile,
    SkinFile,
    NetTransform,
    None,
}
