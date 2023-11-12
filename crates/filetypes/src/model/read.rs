use naia_serde::{BitReader, SerdeInternal as Serde, SerdeErr, UnsignedVariableInteger};

use crate::{model::ModelActionType, ModelAction, common::{ScaleSerdeInt, SerdeQuat, TranslationSerdeInt, NetTransformEntityType}};

impl ModelAction {
    pub fn read(bit_reader: &mut BitReader) -> Result<Vec<Self>, SerdeErr> {
        let mut actions = Vec::new();

        loop {
            let action_type = ModelActionType::de(bit_reader)?;

            match action_type {
                ModelActionType::SkelFile => {
                    let path = String::de(bit_reader)?;
                    let file_name = String::de(bit_reader)?;
                    actions.push(ModelAction::SkelFile(path, file_name));
                }
                ModelActionType::SkinFile => {
                    let path = String::de(bit_reader)?;
                    let file_name = String::de(bit_reader)?;
                    let file_type = NetTransformEntityType::de(bit_reader)?;
                    actions.push(ModelAction::SkinOrSceneFile(path, file_name, file_type));
                }
                ModelActionType::NetTransform => {
                    let skin_index: u16 = UnsignedVariableInteger::<6>::de(bit_reader)?.to();

                    let vertex_name = String::de(bit_reader)?;

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

                    actions.push(ModelAction::NetTransform(
                        skin_index,
                        vertex_name,
                        translation_x,
                        translation_y,
                        translation_z,
                        scale_x,
                        scale_y,
                        scale_z,
                        rotation,
                    ));
                }
                ModelActionType::None => {
                    break;
                }
            }
        }

        Ok(actions)
    }
}