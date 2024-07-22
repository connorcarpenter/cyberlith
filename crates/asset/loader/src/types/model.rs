use logging::info;

use asset_serde::bits::ComponentFileType;
use math::{Quat, Vec3};
use render_api::components::Transform;

use crate::{
    asset_dependency::{AssetComponent, AssetComponentHandle, AssetDependency},
    finish_component_dependency, AssetHandle, SceneData, SkeletonData, SkinData, TypedAssetId,
};

pub struct ModelData {
    skeleton_file: AssetDependency<SkeletonData>,
    component_files: Vec<AssetComponent>,
    net_transforms: Vec<(usize, String, Transform)>,
    computed_components: Option<Vec<(AssetComponentHandle, Transform)>>,
}

impl Default for ModelData {
    fn default() -> Self {
        panic!("");
    }
}

impl ModelData {
    pub(crate) fn load_dependencies(
        &self,
        handle: AssetHandle<Self>,
        dependencies: &mut Vec<(TypedAssetId, TypedAssetId)>,
    ) {
        {
            let AssetDependency::<SkeletonData>::AssetId(asset_id) = &self.skeleton_file else {
                panic!("expected path right after load");
            };
            dependencies.push((handle.into(), TypedAssetId::Skeleton(asset_id.clone())));
        }

        for file in self.component_files.iter() {
            match file {
                AssetComponent::Skin(AssetDependency::<SkinData>::AssetId(asset_id)) => {
                    dependencies.push((handle.into(), TypedAssetId::Skin(asset_id.clone())));
                }
                AssetComponent::Scene(AssetDependency::<SceneData>::AssetId(asset_id)) => {
                    dependencies.push((handle.into(), TypedAssetId::Scene(asset_id.clone())));
                }
                _ => {
                    panic!("expected unloaded (no handles!) skin or scene file");
                }
            }
        }
    }

    pub(crate) fn finish_dependency(&mut self, dependency_typed_id: TypedAssetId) {
        match dependency_typed_id {
            TypedAssetId::Skeleton(asset_id) => {
                let asset_handle = AssetHandle::<SkeletonData>::new(asset_id);
                self.skeleton_file.load_asset_handle(asset_handle);
            }
            TypedAssetId::Skin(asset_id) => {
                let asset_handle = AssetHandle::<SkinData>::new(asset_id);
                let component_handle = AssetComponentHandle::Skin(asset_handle);
                finish_component_dependency(&mut self.component_files, component_handle);
            }
            TypedAssetId::Scene(asset_id) => {
                let asset_handle = AssetHandle::<SceneData>::new(asset_id);
                let component_handle = AssetComponentHandle::Scene(asset_handle);
                finish_component_dependency(&mut self.component_files, component_handle);
            }
            _ => {
                panic!("unexpected type of handle");
            }
        }
    }

    pub(crate) fn all_dependencies_loaded(&self) -> bool {
        // check skeleton
        let AssetDependency::<SkeletonData>::AssetHandle(_) = &self.skeleton_file else {
            return false;
        };

        // check components
        for file in self.component_files.iter() {
            match file {
                AssetComponent::Skin(AssetDependency::<SkinData>::AssetHandle(_)) => {}
                AssetComponent::Scene(AssetDependency::<SceneData>::AssetHandle(_)) => {}
                _ => {
                    return false;
                }
            }
        }
        true
    }

    pub fn get_skeleton_handle(&self) -> AssetHandle<SkeletonData> {
        if let AssetDependency::<SkeletonData>::AssetHandle(handle) = &self.skeleton_file {
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

            let file = &self.component_files[*file_index];
            match file {
                AssetComponent::Skin(AssetDependency::<SkinData>::AssetHandle(handle)) => {
                    components.push((AssetComponentHandle::Skin(*handle), final_transform));
                }
                AssetComponent::Scene(AssetDependency::<SceneData>::AssetHandle(handle)) => {
                    components.push((AssetComponentHandle::Scene(*handle), final_transform));
                }
                _ => panic!("checking for all dependencies loaded should have caught this!"),
            }
        }
        self.computed_components = Some(components);
    }

    pub fn get_components_ref(&self) -> Option<&Vec<(AssetComponentHandle, Transform)>> {
        self.computed_components.as_ref()
    }

    pub(crate) fn get_components_copied(&self) -> Vec<(AssetComponentHandle, String, Transform)> {
        let mut output = Vec::new();

        for (file_index, bone_name, transform) in self.net_transforms.iter() {
            let file = &self.component_files[*file_index];
            let skin_or_scene_handle = match file {
                AssetComponent::Skin(AssetDependency::<SkinData>::AssetHandle(handle)) => {
                    AssetComponentHandle::Skin(*handle)
                }
                AssetComponent::Scene(AssetDependency::<SceneData>::AssetHandle(handle)) => {
                    AssetComponentHandle::Scene(*handle)
                }
                _ => {
                    continue;
                }
            };
            output.push((skin_or_scene_handle, bone_name.clone(), *transform));
        }

        output
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        let actions = asset_serde::bits::ModelAction::read(bytes).expect("unable to parse file");

        let mut skel_file_opt = None;
        let mut component_files = Vec::new();
        let mut net_transforms = Vec::new();
        let mut file_index = 0;
        for action in actions {
            match action {
                asset_serde::bits::ModelAction::SkelFile(asset_id) => {
                    // info!("SkelFile: {:?}", asset_id);
                    skel_file_opt = Some(asset_id);
                }
                asset_serde::bits::ModelAction::Component(asset_id, file_type) => {
                    // info!(
                    //     "ComponentFile {} : {:?}. Type: {:?}",
                    //     file_index, asset_id, file_type
                    // );

                    let asset_dependency = match file_type {
                        ComponentFileType::Skin => {
                            AssetComponent::Skin(AssetDependency::<SkinData>::AssetId(asset_id))
                        }
                        ComponentFileType::Scene => {
                            AssetComponent::Scene(AssetDependency::<SceneData>::AssetId(asset_id))
                        }
                    };

                    component_files.push(asset_dependency);

                    file_index += 1;
                }
                asset_serde::bits::ModelAction::NetTransform(
                    file_index,
                    name,
                    x,
                    y,
                    z,
                    scale_x,
                    scale_y,
                    scale_z,
                    rotation,
                ) => {
                    // info!("NetTransform {} : {}, position ({} {} {}), scale: ({} {} {}), rotation: ({}, {}, {}, {})",
                    //          file_index,
                    //          name,
                    //          x, y, z,
                    //          scale_x, scale_y, scale_z,
                    //          rotation.x, rotation.y, rotation.z, rotation.w);
                    let transform =
                        Transform::from_translation(Vec3::new(x as f32, y as f32, z as f32))
                            .with_scale(Vec3::new(scale_x, scale_y, scale_z))
                            .with_rotation(Quat::from_xyzw(
                                rotation.x, rotation.y, rotation.z, rotation.w,
                            ));
                    net_transforms.push((file_index as usize, name, transform));
                }
            }
        }

        Self {
            skeleton_file: AssetDependency::AssetId(skel_file_opt.unwrap()),
            component_files,
            net_transforms,
            computed_components: None,
        }
    }
}
