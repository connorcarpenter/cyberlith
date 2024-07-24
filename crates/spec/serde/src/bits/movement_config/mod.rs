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

pub type VelocitySerdeInt = UnsignedVariableInteger<4>;

use naia_serde::UnsignedVariableInteger;

pub struct MovementConfigBits {
    max_velocity: f32,
}

impl MovementConfigBits {
    pub fn new(max_velocity: f32) -> Self {
        Self { max_velocity }
    }

    pub fn get_max_velocity(&self) -> f32 {
        self.max_velocity
    }
}
