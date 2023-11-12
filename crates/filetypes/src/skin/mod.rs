
use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature = "read")] {
        mod read;
        pub use read::*;
    } else {}
}

cfg_if! {
    if #[cfg(feature = "write")] {
        mod write;
        pub use write::*;
    } else {}
}

use naia_serde::SerdeInternal as Serde;

#[derive(Clone)]
pub enum SkinAction {
    // path, file_name
    PaletteFile(String, String),
    // path, file_name
    MeshFile(String, String),
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