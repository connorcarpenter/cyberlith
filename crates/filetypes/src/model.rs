
use naia_serde::{BitReader, FileBitWriter, SerdeInternal as Serde, SerdeErr, SignedVariableInteger, UnsignedVariableInteger};

use crate::common::{NetTransformEntityType, SerdeQuat};

// Actions
#[derive(Clone)]
enum ModelAction {
    // path, file_name
    SkelFile(String, String),
    SkinOrSceneFile(String, String, NetTransformEntityType),
    NetTransform(u16, String, i16, i16, i16, f32, f32, f32, SerdeQuat),
}

#[derive(Serde, Clone, PartialEq)]
enum ModelActionType {
    SkelFile,
    SkinFile,
    NetTransform,
    None,
}

type TranslationSerdeInt = SignedVariableInteger<4>;
type ScaleSerdeInt = UnsignedVariableInteger<4>;

// Writer
pub struct ModelWriter;

impl ModelWriter {

    fn write_from_actions(&self, actions: Vec<ModelAction>) -> Box<[u8]> {
        let mut bit_writer = FileBitWriter::new();

        for action in actions {
            match action {
                ModelAction::SkelFile(path, file_name) => {
                    ModelActionType::SkelFile.ser(&mut bit_writer);
                    path.ser(&mut bit_writer);
                    file_name.ser(&mut bit_writer);
                }
                ModelAction::SkinOrSceneFile(path, file_name, file_type) => {
                    ModelActionType::SkinFile.ser(&mut bit_writer);
                    path.ser(&mut bit_writer);
                    file_name.ser(&mut bit_writer);
                    file_type.ser(&mut bit_writer);
                }
                ModelAction::NetTransform(
                    skin_index,
                    vertex_name,
                    translation_x,
                    translation_y,
                    translation_z,
                    scale_x,
                    scale_y,
                    scale_z,
                    rotation,
                ) => {
                    ModelActionType::NetTransform.ser(&mut bit_writer);

                    UnsignedVariableInteger::<6>::new(skin_index).ser(&mut bit_writer);

                    vertex_name.ser(&mut bit_writer);

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
        ModelActionType::None.ser(&mut bit_writer);

        bit_writer.to_bytes()
    }
}

// Reader
pub struct ModelReader;

impl ModelReader {
    fn read_to_actions(bit_reader: &mut BitReader) -> Result<Vec<ModelAction>, SerdeErr> {
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
