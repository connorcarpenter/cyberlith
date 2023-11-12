
use naia_serde::{BitReader, FileBitWriter, SerdeInternal as Serde, SerdeErr, SignedVariableInteger, UnsignedVariableInteger};

use crate::common::{NetTransformEntityType, SerdeQuat};

// Actions
#[derive(Clone)]
enum SceneAction {
    SkinOrSceneFile(String, String, NetTransformEntityType),
    NetTransform(u16, i16, i16, i16, f32, f32, f32, SerdeQuat),
}

#[derive(Serde, Clone, PartialEq)]
enum SceneActionType {
    SkinOrSceneFile,
    NetTransform,
    None,
}

pub type TranslationSerdeInt = SignedVariableInteger<4>;
pub type ScaleSerdeInt = UnsignedVariableInteger<4>;

// Writer
pub struct SceneWriter;

impl SceneWriter {

    fn write_from_actions(&self, actions: Vec<SceneAction>) -> Box<[u8]> {
        let mut bit_writer = FileBitWriter::new();

        for action in actions {
            match action {
                SceneAction::SkinOrSceneFile(path, file_name, file_type) => {
                    SceneActionType::SkinOrSceneFile.ser(&mut bit_writer);
                    path.ser(&mut bit_writer);
                    file_name.ser(&mut bit_writer);
                    file_type.ser(&mut bit_writer);
                }
                SceneAction::NetTransform(
                    skin_index,
                    translation_x,
                    translation_y,
                    translation_z,
                    scale_x,
                    scale_y,
                    scale_z,
                    rotation,
                ) => {
                    SceneActionType::NetTransform.ser(&mut bit_writer);

                    UnsignedVariableInteger::<6>::new(skin_index).ser(&mut bit_writer);

                    let translation_x = TranslationSerdeInt::new(translation_x);
                    let translation_y = TranslationSerdeInt::new(translation_y);
                    let translation_z = TranslationSerdeInt::new(translation_z);

                    translation_x.ser(&mut bit_writer);
                    translation_y.ser(&mut bit_writer);
                    translation_z.ser(&mut bit_writer);

                    let scale_x = ScaleSerdeInt::new((scale_x * 100.0) as u32);
                    let scale_y = ScaleSerdeInt::new((scale_y * 100.0) as u32);
                    let scale_z = ScaleSerdeInt::new((scale_z * 100.0) as u32);

                    scale_x.ser(&mut bit_writer);
                    scale_y.ser(&mut bit_writer);
                    scale_z.ser(&mut bit_writer);

                    rotation.ser(&mut bit_writer);
                }
            }
        }

        // continue bit
        SceneActionType::None.ser(&mut bit_writer);

        bit_writer.to_bytes()
    }
}

// Reader
pub struct SceneReader;

impl SceneReader {
    fn read_to_actions(bit_reader: &mut BitReader) -> Result<Vec<SceneAction>, SerdeErr> {
        let mut actions = Vec::new();

        loop {
            let action_type = SceneActionType::de(bit_reader)?;

            match action_type {
                SceneActionType::SkinOrSceneFile => {
                    let path = String::de(bit_reader)?;
                    let file_name = String::de(bit_reader)?;
                    let file_type = NetTransformEntityType::de(bit_reader)?;
                    actions.push(SceneAction::SkinOrSceneFile(path, file_name, file_type));
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

                    actions.push(SceneAction::NetTransform(
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
