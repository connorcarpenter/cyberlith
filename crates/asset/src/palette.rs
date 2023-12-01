use std::fs;

use naia_serde::BitReader;

use render_api::{AssetHash};

#[derive(Hash)]
pub struct PaletteFile {
    path: String,
}

impl AssetHash<PaletteData> for PaletteFile {}

impl PaletteFile {
    pub fn load(path: &str) -> Self {
        Self {
            path: path.to_string(),
        }
    }
}


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

impl From<PaletteFile> for PaletteData {
    fn from(file: PaletteFile) -> Self {
        let file_path = format!("assets/{}", &file.path);

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