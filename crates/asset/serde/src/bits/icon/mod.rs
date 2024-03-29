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

use asset_id::AssetId;
use naia_serde::SerdeInternal as Serde;

#[derive(Debug, Clone)]
pub enum IconFrameAction {
    //////// x, y//
    Vertex(i16, i16),
    //// order_index, palette color index, id1, id2, id3 // (vertex ids) // id4, id5, id6 (edge ids) // TODO: remove order_index?
    Face(u16, u8, u16, u16, u16),
}

#[derive(Serde, Clone, PartialEq)]
enum IconFrameActionType {
    None,
    Vertex,
    Face,
}

// Actions
#[derive(Debug, Clone)]
pub enum IconAction {
    PaletteFile(AssetId),
    // frame
    Frame(Vec<IconFrameAction>),
}

#[derive(Serde, Clone, PartialEq)]
enum IconActionType {
    None,
    PaletteFile,
    Frame,
}
