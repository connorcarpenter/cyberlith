use naia_serde::{BitReader, SerdeErr, SerdeInternal as Serde};

use spec::Unit;

use crate::bits::unit::UnitBits;

impl UnitBits {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, SerdeErr> {
        let mut bit_reader = BitReader::new(bytes);
        let bit_reader = &mut bit_reader;

        Self::de(bit_reader)
    }
}

impl Into<Unit> for UnitBits {
    fn into(self) -> Unit {
        Unit::new(
            self.get_animated_model_asset_id(),
            self.get_movement_config_asset_id()
        )
    }
}