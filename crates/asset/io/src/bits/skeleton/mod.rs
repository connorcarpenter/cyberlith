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

use crate::bits::common::SerdeRotation;

#[derive(Debug)]
pub enum SkelAction {
    //////// x,   y,   z, Option<parent_id, angle>, vertex_name //
    Vertex(i16, i16, i16, Option<(u16, SerdeRotation)>, Option<String>),
}
