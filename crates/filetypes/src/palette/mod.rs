
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

use naia_serde::{SerdeInternal as Serde};

// Actions
#[derive(Clone)]
pub enum PaletteAction {
    // red, green, blue
    Color(u8, u8, u8),
}

#[derive(Serde, Clone, PartialEq)]
pub enum PaletteActionType {
    Color,
    None,
}