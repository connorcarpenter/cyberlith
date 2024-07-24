use naia_serde::{FileBitWriter, SerdeInternal as Serde};

use crate::bits::{movement_config::VelocitySerdeInt, MovementConfigBits};

impl Into<Vec<u8>> for MovementConfigBits {
    fn into(self) -> Vec<u8> {
        let mut bit_writer = FileBitWriter::new();

        let max_velocity = (self.get_max_velocity() * 100.0) as u32;
        let max_velocity = VelocitySerdeInt::new(max_velocity);
        max_velocity.ser(&mut bit_writer);

        bit_writer.to_vec()
    }
}
