use naia_serde::{FileBitWriter, SerdeInternal as Serde};

use crate::bits::{palette::PaletteActionType, PaletteAction};

impl PaletteAction {
    pub fn write(actions: Vec<Self>) -> Box<[u8]> {
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
