use naia_serde::{BitReader, SerdeErr, SerdeInternal as Serde};

use crate::{palette::PaletteActionType, PaletteAction};

impl PaletteAction {
    pub fn read(bytes: &[u8]) -> Result<Vec<Self>, SerdeErr> {
        let mut bit_reader = BitReader::new(bytes);
        let bit_reader = &mut bit_reader;
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
