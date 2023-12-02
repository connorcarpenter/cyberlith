use std::fs;

use naia_serde::BitReader;

use filetypes::FileTransformEntityType;
use math::{Quat, Vec3};
use render_api::{AssetHash, components::Transform, Handle};
use crate::asset_dependency::{AssetDependency, SkinOrScene};

use crate::{AssetHandle, SkinData};

impl AssetHash<SceneData> for String {}

pub struct SceneData {
    skin_or_scene_files: Vec<SkinOrScene>,
    net_transforms: Vec<(usize, Transform)>,
}

impl Default for SceneData {
    fn default() -> Self {
        panic!("");
    }
}

impl SceneData {
    pub fn load_dependencies(&self, handle: Handle<Self>, dependencies: &mut Vec<(AssetHandle, String)>) {
        for file in self.skin_or_scene_files.iter() {
            match file {
                SkinOrScene::Skin(AssetDependency::<SkinData>::Path(path)) => {
                    dependencies.push((handle.into(), path.clone()));
                }
                SkinOrScene::Scene(AssetDependency::<SceneData>::Path(path)) => {
                    dependencies.push((handle.into(), path.clone()));
                }
                _ => {
                    panic!("expected unloaded (no handles!) skin or scene file");
                }
            }
        }
    }
}

impl From<String> for SceneData {
    fn from(path: String) -> Self {
        let file_path = format!("assets/{}", path);

        let Ok(data) = fs::read(&file_path) else {
            panic!("unable to read file: {:?}", &file_path);
        };

        let mut bit_reader = BitReader::new(&data);

        let actions =
            filetypes::SceneAction::read(&mut bit_reader).expect("unable to parse file");

        let mut skin_or_scene_files = Vec::new();
        let mut net_transforms = Vec::new();
        let mut file_index = 0;
        for action in actions {
            match action {
                filetypes::SceneAction::SkinOrSceneFile(path, name, file_type) => {
                    println!("SkinOrSceneFile {} : {}/{}. Type: {:?}", file_index, path, name, file_type);

                    let asset_dependency = match file_type {
                        FileTransformEntityType::Skin => {
                            SkinOrScene::Skin(AssetDependency::<SkinData>::Path(format!("{}/{}", path, name)))
                        }
                        FileTransformEntityType::Scene => {
                            SkinOrScene::Scene(AssetDependency::<SceneData>::Path(format!("{}/{}", path, name)))
                        }
                    };

                    skin_or_scene_files.push(asset_dependency);

                    file_index += 1;
                }
                filetypes::SceneAction::NetTransform(file_index, x, y, z, scale_x, scale_y, scale_z, rotation) => {
                    println!("NetTransform {} : position ({} {} {}), scale: ({} {} {}), rotation: ({}, {}, {}, {})",
                             file_index,
                             x, y, z,
                             scale_x, scale_y, scale_z,
                             rotation.x, rotation.y, rotation.z, rotation.w);
                    let transform = Transform::from_translation(Vec3::new(x as f32, y as f32, z as f32))
                        .with_scale(Vec3::new(scale_x, scale_y, scale_z))
                        .with_rotation(Quat::from_xyzw(rotation.x, rotation.y, rotation.z, rotation.w));
                    net_transforms.push((file_index as usize, transform));
                }
            }
        }

        // todo: lots here

        Self {
            skin_or_scene_files,
            net_transforms,
        }
    }
}