use std::fs;

use naia_serde::BitReader;

use math::Vec3;
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

        let skel_actions =
            filetypes::SkelAction::read(&mut bit_reader).expect("unable to read skel file");


        let mut vertices = Vec::new();

        for action in skel_actions {
            match action {
                filetypes::SkelAction::Vertex(x, y, z, parent_opt, name_opt) => {
                    println!("Vertex: ({}, {}, {}), parent: {:?}, name: {:?}", x, y, z, parent_opt, name_opt);
                    let vertex = Vec3::new(x as f32, y as f32, z as f32);
                    vertices.push(vertex);
                }
            }
        }

        // todo: lots here

        Self {

        }
    }
}