use naia_serde::{BitReader, SerdeInternal as Serde, SerdeErr};

use crate::{palette::PaletteActionType, PaletteAction};

impl PaletteAction {
    pub fn read(bit_reader: &mut BitReader) -> Result<Vec<Self>, SerdeErr> {
        let mut actions = Vec::new();

        loop {
            let action_type = PaletteActionType::de(bit_reader)?;

            match action_type {
                PaletteActionType::Color => {
                    let r = u8::de(bit_reader)?;
                    let g = u8::de(bit_reader)?;
                    let b = u8::de(bit_reader)?;
                    actions.push(Self::Color(r, g, b));
                }
                PaletteActionType::None => {
                    break;
                }
            }
        }

        Ok(actions)
    }
}