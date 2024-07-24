use naia_serde::{BitReader, SerdeErr, SerdeInternal as Serde};

use spec::MovementConfig;

use crate::bits::{MovementConfigBits, movement_config::VelocitySerdeInt};

/////

impl MovementConfigBits {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, SerdeErr> {
        let mut bit_reader = BitReader::new(bytes);
        let bit_reader = &mut bit_reader;

        let max_velocity: u32 = VelocitySerdeInt::de(bit_reader)?.to();
        let max_velocity = (max_velocity as f32) / 100.0;

        Ok(Self::new(max_velocity))
    }
}

impl Into<MovementConfig> for MovementConfigBits {
    fn into(self) -> MovementConfig {
        MovementConfig::new(self.get_max_velocity())
    }
}