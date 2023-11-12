use naia_serde::{BitReader, SerdeInternal as Serde, SerdeErr};

use crate::{skin::SkinActionType, SkinAction};

impl SkinAction {
    pub fn read(bit_reader: &mut BitReader) -> Result<Vec<Self>, SerdeErr> {
        let mut actions = Vec::new();

        loop {
            let action_type = SkinActionType::de(bit_reader)?;

            match action_type {
                SkinActionType::PaletteFile => {
                    let path = String::de(bit_reader)?;
                    let file_name = String::de(bit_reader)?;
                    actions.push(Self::PaletteFile(path, file_name));
                }
                SkinActionType::MeshFile => {
                    let path = String::de(bit_reader)?;
                    let file_name = String::de(bit_reader)?;
                    actions.push(Self::MeshFile(path, file_name));
                }
                SkinActionType::BackgroundColor => {
                    let palette_color_index = u8::de(bit_reader)?;
                    actions.push(Self::BackgroundColor(palette_color_index));
                }
                SkinActionType::SkinColor => {
                    let face_index = u16::de(bit_reader)?;
                    let palette_color_index = u8::de(bit_reader)?;
                    actions.push(Self::SkinColor(face_index, palette_color_index));
                }
                SkinActionType::None => {
                    break;
                }
            }
        }

        Ok(actions)
    }
}