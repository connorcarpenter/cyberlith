use naia_serde::{FileBitWriter, SerdeInternal as Serde};

use spec::AnimatedModel;

use crate::bits::AnimatedModelBits;

impl From<&AnimatedModel> for AnimatedModelBits {
    fn from(value: &AnimatedModel) -> Self {
        let mut me = Self::new(value.get_model_asset_id());

        for (name, asset_id) in value.get_animations() {
            me.add_animation(name, *asset_id);
        }

        me
    }
}

impl Into<Vec<u8>> for AnimatedModelBits {
    fn into(self) -> Vec<u8> {
        let mut bit_writer = FileBitWriter::new();

        self.ser(&mut bit_writer);

        bit_writer.to_vec()
    }
}
