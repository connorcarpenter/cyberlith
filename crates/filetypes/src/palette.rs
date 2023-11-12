
use naia_serde::{
    BitReader, FileBitWriter, SerdeInternal as Serde, SerdeErr,
};

// Actions
#[derive(Clone)]
enum PaletteAction {
    // red, green, blue
    Color(u8, u8, u8),
}

#[derive(Serde, Clone, PartialEq)]
enum PaletteActionType {
    Color,
    None,
}

// Writer
pub struct PaletteWriter;

impl PaletteWriter {

    fn write_from_actions(&self, actions: Vec<PaletteAction>) -> Box<[u8]> {
        let mut bit_writer = FileBitWriter::new();

        for action in actions {
            match action {
                PaletteAction::Color(r, g, b) => {
                    PaletteActionType::Color.ser(&mut bit_writer);
                    r.ser(&mut bit_writer);
                    g.ser(&mut bit_writer);
                    b.ser(&mut bit_writer);
                }
            }
        }

        // continue bit
        PaletteActionType::None.ser(&mut bit_writer);

        bit_writer.to_bytes()
    }
}

// Reader
pub struct PaletteReader;

impl PaletteReader {
    fn read_to_actions(bit_reader: &mut BitReader) -> Result<Vec<PaletteAction>, SerdeErr> {
        let mut actions = Vec::new();

        loop {
            let action_type = PaletteActionType::de(bit_reader)?;

            match action_type {
                PaletteActionType::Color => {
                    let r = u8::de(bit_reader)?;
                    let g = u8::de(bit_reader)?;
                    let b = u8::de(bit_reader)?;
                    actions.push(PaletteAction::Color(r, g, b));
                }
                PaletteActionType::None => {
                    break;
                }
            }
        }

        Ok(actions)
    }
}
