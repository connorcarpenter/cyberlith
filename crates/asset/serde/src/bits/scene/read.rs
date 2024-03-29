use asset_id::AssetId;
use naia_serde::{BitReader, SerdeErr, SerdeInternal as Serde, UnsignedVariableInteger};

use crate::bits::{
    common::{ComponentFileType, ScaleSerdeInt, SerdeQuat, TranslationSerdeInt},
    scene::SceneActionType,
    SceneAction,
};

impl SceneAction {
    pub fn read(bytes: &[u8]) -> Result<Vec<Self>, SerdeErr> {
        let mut bit_reader = BitReader::new(bytes);
        let bit_reader = &mut bit_reader;
        let mut actions = Vec::new();

        loop {
            let action_type = SceneActionType::de(bit_reader)?;

            match action_type {
                SceneActionType::ComponentFile => {
                    let val = u32::de(bit_reader)?;
                    let asset_id = AssetId::from_u32(val).unwrap();
                    let file_type = ComponentFileType::de(bit_reader)?;
                    actions.push(Self::Component(asset_id, file_type));
                }
                SceneActionType::NetTransform => {
                    let skin_index: u16 = UnsignedVariableInteger::<6>::de(bit_reader)?.to();

                    let translation_x = TranslationSerdeInt::de(bit_reader)?.to();
                    let translation_y = TranslationSerdeInt::de(bit_reader)?.to();
                    let translation_z = TranslationSerdeInt::de(bit_reader)?.to();

                    let scale_x: u32 = ScaleSerdeInt::de(bit_reader)?.to();
                    let scale_y: u32 = ScaleSerdeInt::de(bit_reader)?.to();
                    let scale_z: u32 = ScaleSerdeInt::de(bit_reader)?.to();
                    let scale_x = (scale_x as f32) / 100.0;
                    let scale_y = (scale_y as f32) / 100.0;
                    let scale_z = (scale_z as f32) / 100.0;

                    let rotation = SerdeQuat::de(bit_reader)?;

                    actions.push(Self::NetTransform(
                        skin_index,
                        translation_x,
                        translation_y,
                        translation_z,
                        scale_x,
                        scale_y,
                        scale_z,
                        rotation,
                    ));
                }
                SceneActionType::None => {
                    break;
                }
            }
        }

        Ok(actions)
    }
}
