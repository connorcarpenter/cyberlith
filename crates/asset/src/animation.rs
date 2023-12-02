use std::fs;

use naia_serde::BitReader;

use render_api::{AssetHash, Handle};

use crate::AssetHandle;

impl AssetHash<AnimationData> for String {}

pub struct AnimationData {
    skeleton_file: String,
}

impl Default for AnimationData {
    fn default() -> Self {
        panic!("");
    }
}

impl AnimationData {
    pub fn load_dependencies(&self, handle: Handle<Self>, dependencies: &mut Vec<(AssetHandle, String)>) {
        dependencies.push((handle.into(), self.skeleton_file.clone()));
    }
}

impl From<String> for AnimationData {
    fn from(path: String) -> Self {
        let file_path = format!("assets/{}", path);

        let Ok(data) = fs::read(&file_path) else {
            panic!("unable to read file: {:?}", &file_path);
        };

        let mut bit_reader = BitReader::new(&data);

        let actions =
            filetypes::AnimAction::read(&mut bit_reader).expect("unable to parse file");

        let mut skel_file_opt = None;
        for action in actions {
            match action {
                filetypes::AnimAction::SkelFile(path, file_name) => {
                    println!("SkelFile: {}/{}", path, file_name);
                    skel_file_opt = Some(format!("{}/{}", path, file_name));
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
            skeleton_file: skel_file_opt.unwrap(),
        }
    }
}