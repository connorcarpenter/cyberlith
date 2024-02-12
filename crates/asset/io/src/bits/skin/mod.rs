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

use crate::json::AssetId;

#[derive(Clone)]
pub enum SkinAction {
    // path, file_name
    PaletteFile(AssetId),
    // path, file_name
    MeshFile(AssetId),
    // palette color index
    BackgroundColor(u8),
    // mesh face index, palette color index
    SkinColor(u16, u8),
}

#[derive(Serde, Clone, PartialEq)]
pub enum SkinActionType {
    PaletteFile,
    MeshFile,
    BackgroundColor,
    SkinColor,
    None,
}
