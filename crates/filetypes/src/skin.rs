
use naia_serde::{BitReader, FileBitWriter, SerdeInternal as Serde, SerdeErr};

// Actions
#[derive(Clone)]
enum SkinAction {
    // path, file_name
    PaletteFile(String, String),
    // path, file_name
    MeshFile(String, String),
    // palette color index
    BackgroundColor(u8),
    // mesh face index, palette color index
    SkinColor(u16, u8),
}

#[derive(Serde, Clone, PartialEq)]
enum SkinActionType {
    PaletteFile,
    MeshFile,
    BackgroundColor,
    SkinColor,
    None,
}

// Writer
pub struct SkinWriter;

impl SkinWriter {

    fn write_from_actions(&self, actions: Vec<SkinAction>) -> Box<[u8]> {
        let mut bit_writer = FileBitWriter::new();

        for action in actions {
            match action {
                SkinAction::PaletteFile(path, file_name) => {
                    SkinActionType::PaletteFile.ser(&mut bit_writer);
                    path.ser(&mut bit_writer);
                    file_name.ser(&mut bit_writer);
                }
                SkinAction::MeshFile(path, file_name) => {
                    SkinActionType::MeshFile.ser(&mut bit_writer);
                    path.ser(&mut bit_writer);
                    file_name.ser(&mut bit_writer);
                }
                SkinAction::BackgroundColor(palette_color_index) => {
                    SkinActionType::BackgroundColor.ser(&mut bit_writer);

                    // TODO: could optimize these a bit more .. unlikely to use all these bits
                    palette_color_index.ser(&mut bit_writer);
                }
                SkinAction::SkinColor(face_index, palette_color_index) => {
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

// Reader
pub struct SkinReader;

impl SkinReader {
    fn read_to_actions(bit_reader: &mut BitReader) -> Result<Vec<SkinAction>, SerdeErr> {
        let mut actions = Vec::new();

        loop {
            let action_type = SkinActionType::de(bit_reader)?;

            match action_type {
                SkinActionType::PaletteFile => {
                    let path = String::de(bit_reader)?;
                    let file_name = String::de(bit_reader)?;
                    actions.push(SkinAction::PaletteFile(path, file_name));
                }
                SkinActionType::MeshFile => {
                    let path = String::de(bit_reader)?;
                    let file_name = String::de(bit_reader)?;
                    actions.push(SkinAction::MeshFile(path, file_name));
                }
                SkinActionType::BackgroundColor => {
                    let palette_color_index = u8::de(bit_reader)?;
                    actions.push(SkinAction::BackgroundColor(palette_color_index));
                }
                SkinActionType::SkinColor => {
                    let face_index = u16::de(bit_reader)?;
                    let palette_color_index = u8::de(bit_reader)?;
                    actions.push(SkinAction::SkinColor(face_index, palette_color_index));
                }
                SkinActionType::None => {
                    break;
                }
            }
        }

        Ok(actions)
    }
}
