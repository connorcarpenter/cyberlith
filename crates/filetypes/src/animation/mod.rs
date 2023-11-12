
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

use std::collections::HashMap;

use naia_serde::{SerdeInternal as Serde, UnsignedVariableInteger};

use crate::common::SerdeQuat;

// Transition
#[derive(Clone, PartialEq, Serde)]
pub struct Transition {
    duration_5ms: UnsignedVariableInteger<7>,
    //pub easing: Easing,
}

// Actions
pub enum AnimAction {
    // path, file_name
    SkelFile(String, String),
    // shape name -> shape_index
    ShapeIndex(String),
    // shape_index -> rotation
    Frame(HashMap<u16, SerdeQuat>, Transition),
}

#[derive(Serde, Clone, PartialEq)]
pub enum AnimActionType {
    SkelFile,
    ShapeIndex,
    Frame,
    None,
}