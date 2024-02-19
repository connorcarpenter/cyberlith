use bevy_log::info;

use asset_id::AssetId;
use render_api::components::Transform;
use storage::{AssetHash, Handle};
use asset_io::bits::ComponentFileType;
use math::{Quat, Vec3};

use crate::{
    asset_dependency::{AssetDependency, AssetComponent, AssetComponentHandle},
    asset_handle::AssetHandleImpl,
    scene::finish_component_dependency,
    AssetHandle, SceneData, SkeletonData, SkinData,
};

impl AssetHash<ModelData> for String {}

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
        handle: Handle<Self>,
        dependencies: &mut Vec<(AssetHandle, AssetId)>,
    ) {
        {
            let AssetDependency::<SkeletonData>::AssetId(asset_id) = &self.skeleton_file else {
                panic!("expected path right after load");
            };
            dependencies.push((handle.into(), asset_id.clone()));
        }

        for file in self.component_files.iter() {
            match file {
                AssetComponent::Skin(AssetDependency::<SkinData>::AssetId(path)) => {
                    dependencies.push((handle.into(), path.clone()));
                }
                AssetComponent::Scene(AssetDependency::<SceneData>::AssetId(path)) => {
                    dependencies.push((handle.into(), path.clone()));
                }
                _ => {
                    panic!("expected unloaded (no handles!) skin or scene file");
                }
            }
        }
    }

    pub(crate) fn finish_dependency(
        &mut self,
        dependency_id: AssetId,
        dependency_handle: AssetHandle,
    ) {
        match dependency_handle.to_impl() {
            AssetHandleImpl::Skeleton(handle) => {
                self.skeleton_file.load_handle(handle);
            }
            AssetHandleImpl::Skin(handle) => {
                let handle = AssetComponentHandle::Skin(handle);
                finish_component_dependency(
                    &mut self.component_files,
                    dependency_id,
                    handle,
                );
            }
            AssetHandleImpl::Scene(handle) => {
                let handle = AssetComponentHandle::Scene(handle);
                finish_component_dependency(
                    &mut self.component_files,
                    dependency_id,
                    handle,
                );
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
        for file in self.component_files.iter() {
            match file {
                AssetComponent::Skin(AssetDependency::<SkinData>::Handle(_)) => {}
                AssetComponent::Scene(AssetDependency::<SceneData>::Handle(_)) => {}
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

            let file = &self.component_files[*file_index];
            match file {
                AssetComponent::Skin(AssetDependency::<SkinData>::Handle(handle)) => {
                    components.push((AssetComponentHandle::Skin(*handle), final_transform));
                }
                AssetComponent::Scene(AssetDependency::<SceneData>::Handle(handle)) => {
                    components.push((AssetComponentHandle::Scene(*handle), final_transform));
                }
                _ => panic!("checking for all dependencies loaded should have caught this!"),
            }
        }
        self.computed_components = Some(components);
    }

    pub(crate) fn get_components_ref(&self) -> Option<&Vec<(AssetComponentHandle, Transform)>> {
        self.computed_components.as_ref()
    }

    pub(crate) fn get_components_copied(&self) -> Vec<(AssetComponentHandle, String, Transform)> {
        let mut output = Vec::new();

        for (file_index, bone_name, transform) in self.net_transforms.iter() {
            let file = &self.component_files[*file_index];
            let skin_or_scene_handle = match file {
                AssetComponent::Skin(AssetDependency::<SkinData>::Handle(handle)) => {
                    AssetComponentHandle::Skin(*handle)
                }
                AssetComponent::Scene(AssetDependency::<SceneData>::Handle(handle)) => {
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
}

impl From<String> for ModelData {
    fn from(path: String) -> Self {
        let file_path = format!("assets/{}", path);

        let Ok(data) = web_fs::read(&file_path) else {
            panic!("unable to read file: {:?}", &file_path);
        };

        let actions = asset_io::bits::ModelAction::read(&data).expect("unable to parse file");

        let mut skel_file_opt = None;
        let mut component_files = Vec::new();
        let mut net_transforms = Vec::new();
        let mut file_index = 0;
        for action in actions {
            match action {
                asset_io::bits::ModelAction::SkelFile(asset_id) => {
                    info!("SkelFile: {:?}", asset_id);
                    skel_file_opt = Some(asset_id);
                }
                asset_io::bits::ModelAction::Component(asset_id, file_type) => {
                    info!(
                        "ComponentFile {} : {:?}. Type: {:?}",
                        file_index, asset_id, file_type
                    );

                    let asset_dependency =
                        match file_type {
                            ComponentFileType::Skin => {
                                AssetComponent::Skin(AssetDependency::<SkinData>::AssetId(asset_id))
                            }
                            ComponentFileType::Scene => AssetComponent::Scene(
                                AssetDependency::<SceneData>::AssetId(asset_id),
                            ),
                        };

                    component_files.push(asset_dependency);

                    file_index += 1;
                }
                asset_io::bits::ModelAction::NetTransform(
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
                    info!("NetTransform {} : {}, position ({} {} {}), scale: ({} {} {}), rotation: ({}, {}, {}, {})",
                             file_index,
                             name,
                             x, y, z,
                             scale_x, scale_y, scale_z,
                             rotation.x, rotation.y, rotation.z, rotation.w);
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
            component_files: component_files,
            net_transforms,
            computed_components: None,
        }
    }
}
