use naia_serde::{BitReader, SerdeErr, SerdeInternal as Serde, UnsignedVariableInteger};

use crate::{
    common::{FileTransformEntityType, ScaleSerdeInt, SerdeQuat, TranslationSerdeInt},
    scene::SceneActionType,
    SceneAction,
};

impl SceneAction {
    pub fn read(bit_reader: &mut BitReader) -> Result<Vec<Self>, SerdeErr> {
        let mut actions = Vec::new();

        loop {
            let action_type = SceneActionType::de(bit_reader)?;

            match action_type {
                SceneActionType::SkinOrSceneFile => {
                    let path = String::de(bit_reader)?;
                    let file_name = String::de(bit_reader)?;
                    let file_type = FileTransformEntityType::de(bit_reader)?;
                    actions.push(Self::SkinOrSceneFile(path, file_name, file_type));
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
