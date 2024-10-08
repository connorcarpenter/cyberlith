use naia_serde::{FileBitWriter, Serde, UnsignedVariableInteger};

use crate::bits::{
    common::{ScaleSerdeInt, TranslationSerdeInt},
    model::ModelActionType,
    ModelAction,
};

impl ModelAction {
    pub fn write(actions: Vec<Self>) -> Box<[u8]> {
        let mut bit_writer = FileBitWriter::new();

        for action in actions {
            match action {
                Self::SkelFile(asset_id) => {
                    ModelActionType::SkelFile.ser(&mut bit_writer);
                    asset_id.as_u32().ser(&mut bit_writer);
                }
                Self::Component(asset_id, file_type) => {
                    ModelActionType::ComponentFile.ser(&mut bit_writer);
                    asset_id.as_u32().ser(&mut bit_writer);
                    file_type.ser(&mut bit_writer);
                }
                Self::NetTransform(
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
