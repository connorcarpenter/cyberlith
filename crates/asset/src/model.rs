use std::fs;

use bevy_log::info;

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
    computed_components: Option<Vec<(SkinOrSceneHandle, Transform)>>,
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

    pub(crate) fn all_dependencies_loaded(&self) -> bool {

        // check skeleton
        let AssetDependency::<SkeletonData>::Handle(_) = &self.skeleton_file else {
            return false;
        };

        // check components
        for file in self.skin_or_scene_files.iter() {
            match file {
                SkinOrScene::Skin(AssetDependency::<SkinData>::Handle(_)) => {}
                SkinOrScene::Scene(AssetDependency::<SceneData>::Handle(_)) => {}
                _ => {
                    return false;
                }
            }
        }
        true
    }

    pub(crate) fn get_skeleton_handle(&self) -> Handle<SkeletonData> {
        if let AssetDependency::<SkeletonData>::Handle(handle) = &self.skeleton_file {
            *handle
        } else {
            panic!("expected skeleton handle");
        }
    }

    pub(crate) fn compute_components(&mut self, skeleton_data: &SkeletonData) {
        // compute components
        let mut components = Vec::new();
        for (file_index, bone_name, component_transform) in self.net_transforms.iter() {

            let Some(bone_transform) = skeleton_data.get_bone_transform(bone_name) else {
                panic!("unable to find bone in skeleton of name: {}", bone_name);
            };

            let final_transform = component_transform.multiply(bone_transform);

            let file = &self.skin_or_scene_files[*file_index];
            match file {
                SkinOrScene::Skin(AssetDependency::<SkinData>::Handle(handle)) => {
                    components.push((SkinOrSceneHandle::Skin(*handle), final_transform));
                }
                SkinOrScene::Scene(AssetDependency::<SceneData>::Handle(handle)) => {
                    components.push((SkinOrSceneHandle::Scene(*handle), final_transform));
                }
                _ => panic!("checking for all dependencies loaded should have caught this!"),
            }
        }
        self.computed_components = Some(components);
    }

    pub(crate) fn get_components_ref(&self) -> Option<&Vec<(SkinOrSceneHandle, Transform)>> {
        self.computed_components.as_ref()
    }

    pub(crate) fn get_components_copied(&self) -> Vec<(SkinOrSceneHandle, String, Transform)> {
        let mut output = Vec::new();

        for (file_index, bone_name, transform) in self.net_transforms.iter() {
            let file = &self.skin_or_scene_files[*file_index];
            let skin_or_scene_handle = match file {
                SkinOrScene::Skin(AssetDependency::<SkinData>::Handle(handle)) => {
                    SkinOrSceneHandle::Skin(*handle)
                }
                SkinOrScene::Scene(AssetDependency::<SceneData>::Handle(handle)) => {
                    SkinOrSceneHandle::Scene(*handle)
                }
                _ => {
                    continue;
                }
            };
            output.push((skin_or_scene_handle, bone_name.clone(), *transform));
        }

        output
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
                    info!("SkelFile: {}/{}", path, file_name);
                    skel_file_opt = Some(format!("{}/{}", path, file_name));
                }
                filetypes::ModelAction::SkinOrSceneFile(path, name, file_type) => {
                    info!("SkinOrSceneFile {} : {}/{}. Type: {:?}", file_index, path, name, file_type);

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
                    info!("NetTransform {} : {}, position ({} {} {}), scale: ({} {} {}), rotation: ({}, {}, {}, {})",
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
            computed_components: None,
        }
    }
}