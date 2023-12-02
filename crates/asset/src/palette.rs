use std::fs;

use naia_serde::BitReader;

use render_api::{AssetHash, Handle, base::CpuMaterial};

impl AssetHash<PaletteData> for String {}

pub(crate) enum PaletteColor {
    Raw(u8, u8, u8),
    Material(Handle<CpuMaterial>),
}

pub struct PaletteData {
    colors: Vec<PaletteColor>,
}

impl Default for PaletteData {
    fn default() -> Self {
        panic!("");
    }
}

impl PaletteData {

}

impl From<String> for PaletteData {
    fn from(path: String) -> Self {
        let file_path = format!("assets/{}", path);

        let Ok(data) = fs::read(&file_path) else {
            panic!("unable to read file: {:?}", &file_path);
        };

        let mut bit_reader = BitReader::new(&data);

        let actions =
            filetypes::PaletteAction::read(&mut bit_reader).expect("unable to parse file");

        let mut colors = Vec::new();
        for action in actions {
            match action {
                filetypes::PaletteAction::Color(r, g, b) => {
                    println!("Color: ({}, {}, {})", r, g, b);
                    colors.push(PaletteColor::Raw(r, g, b));
                }
            }
        }

        Self {
            colors
        }
    }
}