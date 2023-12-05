use std::fs;

use bevy_log::info;

use naia_serde::BitReader;

use render_api::{AssetHash, components::Transform};

impl AssetHash<SkeletonData> for String {}

pub struct SkeletonData {

}

impl Default for SkeletonData {
    fn default() -> Self {
        panic!("");
    }
}

impl SkeletonData {
    pub(crate) fn get_bone_transform(&self, bone_name: &str) -> Option<&Transform> {
        todo!()
    }
}

impl From<String> for SkeletonData {
    fn from(path: String) -> Self {
        let file_path = format!("assets/{}", path);

        let Ok(data) = fs::read(&file_path) else {
            panic!("unable to read file: {:?}", &file_path);
        };

        let mut bit_reader = BitReader::new(&data);

        let actions =
            filetypes::SkelAction::read(&mut bit_reader).expect("unable to parse file");

        for action in actions {
            match action {
                filetypes::SkelAction::Vertex(x, y, z, parent_opt, name_opt) => {
                    info!("Vertex: ({}, {}, {}), parent: {:?}, name: {:?}", x, y, z, parent_opt, name_opt);
                }
            }
        }

        // todo: lots here

        Self {

        }
    }
}