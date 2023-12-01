use std::fs;

use naia_serde::BitReader;

use render_api::{AssetHash};

#[derive(Hash)]
pub struct SceneFile {
    path: String,
}

impl AssetHash<SceneData> for SceneFile {}

impl SceneFile {
    pub fn load(path: &str) -> Self {
        Self {
            path: path.to_string(),
        }
    }
}


pub struct SceneData {

}

impl Default for SceneData {
    fn default() -> Self {
        Self {

        }
    }
}

impl SceneData {

}

impl From<SceneFile> for SceneData {
    fn from(file: SceneFile) -> Self {
        let file_path = format!("assets/{}", &file.path);

        let data = fs::read(file_path).expect("unable to read file");

        let mut bit_reader = BitReader::new(&data);

        let actions =
            filetypes::SceneAction::read(&mut bit_reader).expect("unable to parse file");

        let mut file_index = 0;
        for action in actions {
            match action {
                filetypes::SceneAction::SkinOrSceneFile(path, name, file_type) => {
                    println!("SkinOrSceneFile {} : {}/{}. Type: {:?}", file_index, path, name, file_type);

                    file_index += 1;
                }
                filetypes::SceneAction::NetTransform(file_index, x, y, z, scale_x, scale_y, scale_z, rotation) => {
                    println!("NetTransform {} : position ({} {} {}), scale: ({} {} {}), rotation: ({}, {}, {}, {})",
                             file_index,
                             x, y, z,
                             scale_x, scale_y, scale_z,
                             rotation.x, rotation.y, rotation.z, rotation.w);
                }
            }
        }

        // todo: lots here

        Self {

        }
    }
}