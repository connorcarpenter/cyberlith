use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature = "read_bits")] {
        mod read;
    } else {}
}

cfg_if! {
    if #[cfg(feature = "write_bits")] {
        mod write;
    } else {}
}

use naia_serde::SerdeInternal as Serde;

use asset_id::AssetId;

#[derive(Clone)]
pub enum SkinAction {
    // assetid
    PaletteFile(AssetId),
    // assetid
    MeshData(AssetId),
    // palette color index
    BackgroundColor(u8),
    // mesh face index, palette color index
    SkinColor(u16, u8),
}

#[derive(Serde, Clone, PartialEq)]
pub enum SkinActionType {
    PaletteFile,
    MeshData,
    BackgroundColor,
    SkinColor,
    None,
}
