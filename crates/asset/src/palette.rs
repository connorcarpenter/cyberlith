use std::fs;

use naia_serde::BitReader;

use render_api::AssetHash;

impl AssetHash<PaletteData> for String {}

pub struct PaletteData {

}

impl Default for PaletteData {
    fn default() -> Self {
        Self {

        }
    }
}

impl PaletteData {

}

impl From<String> for PaletteData {
    fn from(path: String) -> Self {
        let file_path = format!("assets/{}", path);

        let data = fs::read(file_path).expect("unable to read file");

        let mut bit_reader = BitReader::new(&data);

        let actions =
            filetypes::PaletteAction::read(&mut bit_reader).expect("unable to parse file");

        for action in actions {
            match action {
                filetypes::PaletteAction::Color(r, g, b) => {
                    println!("Color: ({}, {}, {})", r, g, b);
                }
            }
        }

        // todo: lots here

        Self {

        }
    }
}