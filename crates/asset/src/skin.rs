use std::fs;

use naia_serde::BitReader;

use render_api::{AssetHash};

impl AssetHash<SkinData> for String {}

pub struct SkinData {

}

impl Default for SkinData {
    fn default() -> Self {
        Self {

        }
    }
}

impl SkinData {

}

impl From<String> for SkinData {
    fn from(path: String) -> Self {
        let file_path = format!("assets/{}", path);

        let data = fs::read(file_path).expect("unable to read file");

        let mut bit_reader = BitReader::new(&data);

        let actions =
            filetypes::SkinAction::read(&mut bit_reader).expect("unable to parse file");

        for action in actions {
            match action {
                filetypes::SkinAction::PaletteFile(path, file_name) => {
                    println!("PaletteFile: {}/{}", path, file_name);
                }
                filetypes::SkinAction::MeshFile(path, file_name) => {
                    println!("MeshFile: {}/{}", path, file_name);
                }
                filetypes::SkinAction::BackgroundColor(color_index) => {
                    println!("BackgroundColor: {}", color_index);
                }
                filetypes::SkinAction::SkinColor(face_index, color_index) => {
                    println!("SkinColor: face_index: {}, color_index: {}", face_index, color_index);
                }
            }
        }

        // todo: lots here

        Self {

        }
    }
}