use std::fs;

use naia_serde::BitReader;
use filetypes::FileTransformEntityType;
use math::{Quat, Vec3};

use render_api::{AssetHash, components::Transform, Handle};

use crate::{scene::finish_skin_or_scene_dependency, asset_handle::AssetHandleImpl, asset_dependency::{SkinOrSceneHandle, AssetDependency, SkinOrScene}, AssetHandle, SceneData, SkeletonData, SkinData};

impl AssetHash<ModelData> for String {}

pub struct ModelData {
    skeleton_file: AssetDependency<SkeletonData>,
    skin_or_scene_files: Vec<SkinOrScene>,
    net_transforms: Vec<(usize, String, Transform)>,
}

impl Default for ModelData {
    fn default() -> Self {
        panic!("");
    }
}

impl ModelData {
    pub(crate) fn load_dependencies(&self, handle: Handle<Self>, dependencies: &mut Vec<(AssetHandle, String)>) {
        {
            let AssetDependency::<SkeletonData>::Path(path) = &self.skeleton_file else {
                panic!("expected path right after load");
            };
            dependencies.push((handle.into(), path.clone()));
        }

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

    pub(crate) fn finish_dependency(&mut self, dependency_path: String, dependency_handle: AssetHandle) {
        match dependency_handle.to_impl() {
            AssetHandleImpl::Skeleton(handle) => {
                self.skeleton_file.load_handle(handle);
            }
            AssetHandleImpl::Skin(handle) => {
                let handle = SkinOrSceneHandle::Skin(handle);
                finish_skin_or_scene_dependency(&mut self.skin_or_scene_files, dependency_path, handle);
            }
            AssetHandleImpl::Scene(handle) => {
                let handle = SkinOrSceneHandle::Scene(handle);
                finish_skin_or_scene_dependency(&mut self.skin_or_scene_files, dependency_path, handle);
            }
            _ => {
                panic!("unexpected type of handle");
            }
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
            skeleton_file: AssetDependency::Path(skel_file_opt.unwrap()),
            skin_or_scene_files,
            net_transforms,
        }
    }
}