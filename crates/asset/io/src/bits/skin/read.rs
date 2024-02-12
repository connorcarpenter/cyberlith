use naia_serde::{BitReader, SerdeErr, SerdeInternal as Serde};

use crate::bits::{skin::SkinActionType, SkinAction};
use crate::json::AssetId;

impl SkinAction {
    pub fn read(bytes: &[u8]) -> Result<Vec<Self>, SerdeErr> {
        let mut bit_reader = BitReader::new(bytes);
        let bit_reader = &mut bit_reader;
        let mut actions = Vec::new();

        loop {
            let action_type = SkinActionType::de(bit_reader)?;

            match action_type {
                SkinActionType::PaletteFile => {
                    let asset_id_val = u32::de(bit_reader)?;
                    actions.push(Self::PaletteFile(AssetId::from_u32(asset_id_val).unwrap()));
                }
                SkinActionType::MeshFile => {
                    let asset_id_val = u32::de(bit_reader)?;
                    actions.push(Self::MeshFile(AssetId::from_u32(asset_id_val).unwrap()));
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
