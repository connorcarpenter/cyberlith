use naia_serde::{FileBitWriter, SerdeInternal as Serde};

use crate::{skin::SkinActionType, SkinAction};

impl SkinAction {
    pub fn write(&self, actions: Vec<Self>) -> Box<[u8]> {
        let mut bit_writer = FileBitWriter::new();

        for action in actions {
            match action {
                Self::PaletteFile(path, file_name) => {
                    SkinActionType::PaletteFile.ser(&mut bit_writer);
                    path.ser(&mut bit_writer);
                    file_name.ser(&mut bit_writer);
                }
                Self::MeshFile(path, file_name) => {
                    SkinActionType::MeshFile.ser(&mut bit_writer);
                    path.ser(&mut bit_writer);
                    file_name.ser(&mut bit_writer);
                }
                Self::BackgroundColor(palette_color_index) => {
                    SkinActionType::BackgroundColor.ser(&mut bit_writer);

                    // TODO: could optimize these a bit more .. unlikely to use all these bits
                    palette_color_index.ser(&mut bit_writer);
                }
                Self::SkinColor(face_index, palette_color_index) => {
                    SkinActionType::SkinColor.ser(&mut bit_writer);

                    // TODO: could optimize these a bit more .. unlikely to use all these bits
                    face_index.ser(&mut bit_writer);
                    palette_color_index.ser(&mut bit_writer);
                }
            }
        }

        // continue bit
        SkinActionType::None.ser(&mut bit_writer);

        bit_writer.to_bytes()
    }
}