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

use std::collections::HashMap;

use naia_serde::{SerdeInternal as Serde, UnsignedVariableInteger};
use asset_id::AssetId;

use crate::bits::common::SerdeQuat;

// Transition
#[derive(Clone, PartialEq, Serde)]
pub struct Transition {
    duration_5ms: UnsignedVariableInteger<7>,
    //pub easing: Easing,
}

impl Transition {
    pub fn new(duration_ms: u16) -> Self {
        let duration_5ms = duration_ms / 5;
        Self {
            duration_5ms: duration_5ms.into(),
        }
    }

    pub fn get_duration_ms(&self) -> u16 {
        let duration_5ms: u16 = self.duration_5ms.to();
        duration_5ms * 5
    }
}

// Actions
pub enum AnimAction {
    // path, file_name
    SkelFile(AssetId),
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
