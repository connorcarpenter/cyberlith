use std::fs;

use naia_serde::BitReader;
use filetypes::FileTransformEntityType;
use math::{Quat, Vec3};

use render_api::{AssetHash, components::Transform};

impl AssetHash<ModelData> for String {}

pub struct ModelData {
    skeleton_file: String,
    skin_or_scene_files: Vec<(String, String, FileTransformEntityType)>,
    net_transforms: Vec<(usize, String, Transform)>,
}

impl Default for ModelData {
    fn default() -> Self {
        panic!("");
    }
}

impl ModelData {
    pub fn load_dependencies(&self, dependencies: &mut Vec<String>) {
        dependencies.push(self.skeleton_file.clone());

        for (path, file_name, _) in self.skin_or_scene_files.iter() {
            dependencies.push(format!("{}/{}", path, file_name));
        }
    }
}

impl From<String> for ModelData {
    fn from(path: String) -> Self {
        let file_path = format!("assets/{}", path);

        let Ok(data) = fs::read(&file_path) else {
            panic!("unable to read file: {:?}", &file_path);
        };

        let mut bit_reader = BitReader::new(&data);

        let actions =
            filetypes::ModelAction::read(&mut bit_reader).expect("unable to parse file");

        let mut skel_file_opt = None;
        let mut skin_or_scene_files = Vec::new();
        let mut net_transforms = Vec::new();
        let mut file_index = 0;
        for action in actions {
            match action {
                filetypes::ModelAction::SkelFile(path, file_name) => {
                    println!("SkelFile: {}/{}", path, file_name);
                    skel_file_opt = Some(format!("{}/{}", path, file_name));
                }
                filetypes::ModelAction::SkinOrSceneFile(path, name, file_type) => {
                    println!("SkinOrSceneFile {} : {}/{}. Type: {:?}", file_index, path, name, file_type);

                    skin_or_scene_files.push((path, name, file_type));

                    file_index += 1;
                }
                filetypes::ModelAction::NetTransform(file_index, name, x, y, z, scale_x, scale_y, scale_z, rotation) => {
                    println!("NetTransform {} : {}, position ({} {} {}), scale: ({} {} {}), rotation: ({}, {}, {}, {})",
                             file_index,
                             name,
                             x, y, z,
                             scale_x, scale_y, scale_z,
                             rotation.x, rotation.y, rotation.z, rotation.w);
                    let transform = Transform::from_translation(Vec3::new(x as f32, y as f32, z as f32))
                        .with_scale(Vec3::new(scale_x, scale_y, scale_z))
                        .with_rotation(Quat::from_xyzw(rotation.x, rotation.y, rotation.z, rotation.w));
                    net_transforms.push((file_index as usize, name, transform));
                }
            }
        }

        Self {
            skeleton_file: skel_file_opt.unwrap(),
            skin_or_scene_files,
            net_transforms,
        }
    }
}