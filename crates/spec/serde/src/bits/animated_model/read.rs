use naia_serde::{BitReader, SerdeErr, SerdeInternal as Serde};

use spec::AnimatedModel;

use crate::bits::AnimatedModelBits;

impl AnimatedModelBits {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, SerdeErr> {
        let mut bit_reader = BitReader::new(bytes);
        let bit_reader = &mut bit_reader;

        Self::de(bit_reader)
    }
}

impl Into<AnimatedModel> for AnimatedModelBits {
    fn into(self) -> AnimatedModel {
        let mut animated_model = AnimatedModel::new(self.get_model_asset_id());

        for (name, asset_id) in self.get_animations() {
            animated_model.add_animation(name, *asset_id);
        }

        animated_model
    }
}
