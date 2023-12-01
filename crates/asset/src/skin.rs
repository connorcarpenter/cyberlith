use std::fs;

use naia_serde::BitReader;

use render_api::{AssetHash};

#[derive(Hash)]
pub struct SkinFile {
    path: String,
}

impl AssetHash<SkinData> for SkinFile {}

impl SkinFile {
    pub fn load(path: &str) -> Self {
        Self {
            path: path.to_string(),
        }
    }
}


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

impl From<SkinFile> for SkinData {
    fn from(file: SkinFile) -> Self {
        let file_path = format!("assets/{}", &file.path);

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