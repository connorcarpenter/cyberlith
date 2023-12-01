use std::fs;

use naia_serde::BitReader;

use render_api::{AssetHash};

#[derive(Hash)]
pub struct AnimationFile {
    path: String,
}

impl AssetHash<AnimationData> for AnimationFile {}

impl AnimationFile {
    pub fn load(path: &str) -> Self {
        Self {
            path: path.to_string(),
        }
    }
}


pub struct AnimationData {

}

impl Default for AnimationData {
    fn default() -> Self {
        Self {

        }
    }
}

impl AnimationData {

}

impl From<AnimationFile> for AnimationData {
    fn from(file: AnimationFile) -> Self {
        let file_path = format!("assets/{}", &file.path);

        let data = fs::read(file_path).expect("unable to read file");

        let mut bit_reader = BitReader::new(&data);

        let actions =
            filetypes::AnimAction::read(&mut bit_reader).expect("unable to parse file");

        for action in actions {
            match action {
                filetypes::AnimAction::SkelFile(path, file_name) => {
                    println!("SkelFile: {}/{}", path, file_name);
                }
                filetypes::AnimAction::ShapeIndex(name) => {
                    println!("ShapeIndex: {}", name);
                }
                filetypes::AnimAction::Frame(rotation_map, transition_time) => {
                    println!("Frame: {:?}ms", transition_time.get_duration_ms());
                    for (key, value) in rotation_map {
                        println!("index: {} . rotation: ({:?}, {:?}, {:?}, {:?})", key, value.x, value.y, value.z, value.w);
                    }
                }
            }
        }

        // todo: lots here

        Self {

        }
    }
}