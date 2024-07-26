use naia_serde::{FileBitWriter, SerdeInternal as Serde};

use spec::Unit;

use crate::bits::UnitBits;

impl From<&Unit> for UnitBits {
    fn from(value: &Unit) -> Self {
        Self::new(
            value.get_animated_model_asset_id(),
            value.get_movement_config_asset_id(),
        )
    }
}

impl Into<Vec<u8>> for UnitBits {
    fn into(self) -> Vec<u8> {
        let mut bit_writer = FileBitWriter::new();

        self.ser(&mut bit_writer);

        bit_writer.to_vec()
    }
}
