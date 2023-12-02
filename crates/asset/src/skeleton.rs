use std::fs;

use naia_serde::BitReader;

use render_api::{AssetHash};

impl AssetHash<SkeletonData> for String {}

pub struct SkeletonData {

}

impl Default for SkeletonData {
    fn default() -> Self {
        panic!("");
    }
}

impl SkeletonData {

}

impl From<String> for SkeletonData {
    fn from(path: String) -> Self {
        let file_path = format!("assets/{}", path);

        let Ok(data) = fs::read(&file_path) else {
            panic!("unable to read file: {:?}", &file_path);
        };
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