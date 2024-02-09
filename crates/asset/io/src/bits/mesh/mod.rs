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

// Actions
#[derive(Debug, Clone)]
pub enum MeshAction {
    //////// x,   y,   z //
    Vertex(i16, i16, i16),
    //// id1, id2 // (vertex ids)
    Edge(u16, u16),
    //// order_index, id1, id2, id3 // (vertex ids) // id4, id5, id6 (edge ids)
    Face(u16, u16, u16, u16, u16, u16, u16),
}

#[derive(Serde, Clone, PartialEq)]
pub enum MeshActionType {
    None,
    Vertex,
    Edge,
    Face,
}
