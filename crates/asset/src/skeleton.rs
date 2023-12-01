use std::fs;

use naia_serde::BitReader;

use render_api::{AssetHash};

#[derive(Hash)]
pub struct SkeletonFile {
    path: String,
}

impl AssetHash<SkeletonData> for SkeletonFile {}

impl SkeletonFile {
    pub fn load(path: &str) -> Self {
        Self {
            path: path.to_string(),
        }
    }
}


pub struct SkeletonData {

}

impl Default for SkeletonData {
    fn default() -> Self {
        Self {

        }
    }
}

impl SkeletonData {

}

impl From<SkeletonFile> for SkeletonData {
    fn from(file: SkeletonFile) -> Self {
        let file_path = format!("assets/{}", &file.path);

        let data = fs::read(file_path).expect("unable to read file");
        //let data = include_bytes!("cube.mesh");

        let mut bit_reader = BitReader::new(&data);

        let actions =
            filetypes::SkelAction::read(&mut bit_reader).expect("unable to parse file");

        for action in actions {
            match action {
                filetypes::SkelAction::Vertex(x, y, z, parent_opt, name_opt) => {
                    println!("Vertex: ({}, {}, {}), parent: {:?}, name: {:?}", x, y, z, parent_opt, name_opt);
                }
            }
        }

        // todo: lots here

        Self {

        }
    }
}