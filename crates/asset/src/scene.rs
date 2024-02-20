use bevy_log::info;

use asset_id::AssetId;
use asset_io::bits::ComponentFileType;
use math::{Quat, Vec3};
use render_api::components::Transform;
use storage::{StorageHash, Handle};

use crate::{
    asset_dependency::{AssetComponent, AssetComponentHandle, AssetDependency},
    asset_handle::AssetHandleImpl,
    AssetHandle, SkinData,
};

impl StorageHash<SceneData> for String {}

pub struct SceneData {
    component_files: Vec<AssetComponent>,
    net_transforms: Vec<(usize, Transform)>,
    computed_components: Option<Vec<(AssetComponentHandle, Transform)>>,
}

impl Default for SceneData {
    fn default() -> Self {
        panic!("");
    }
}

impl SceneData {
    pub(crate) fn load_dependencies(
        &self,
        handle: Handle<Self>,
        dependencies: &mut Vec<(AssetHandle, AssetId)>,
    ) {
        for file in self.component_files.iter() {
            match file {
                AssetComponent::Skin(AssetDependency::<SkinData>::AssetId(asset_id)) => {
                    dependencies.push((handle.into(), asset_id.clone()));
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
            AssetHandleImpl::Skin(handle) => {
                info!(
                    "finished scene dependency for skin: {}, id: {:?}",
                    &handle.id, dependency_id
                );
                let handle = AssetComponentHandle::Skin(handle);
                finish_component_dependency(&mut self.component_files, dependency_id, handle);
            }
            AssetHandleImpl::Scene(handle) => {
                let handle = AssetComponentHandle::Scene(handle);
                finish_component_dependency(&mut self.component_files, dependency_id, handle);
            }
            _ => {
                panic!("unexpected type of handle");
            }
        }

        if self.all_dependencies_loaded() {
            // compute components
            let mut components = Vec::new();
            for (file_index, transform) in self.net_transforms.iter() {
                let file = &self.component_files[*file_index];
                match file {
                    AssetComponent::Skin(AssetDependency::<SkinData>::Handle(handle)) => {
                        components.push((AssetComponentHandle::Skin(*handle), *transform));
                    }
                    AssetComponent::Scene(AssetDependency::<SceneData>::Handle(handle)) => {
                        components.push((AssetComponentHandle::Scene(*handle), *transform));
                    }
                    _ => panic!("impossible"),
                }
            }
            self.computed_components = Some(components);
        }
    }

    fn all_dependencies_loaded(&self) -> bool {
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

    pub(crate) fn get_components(&self) -> Option<&Vec<(AssetComponentHandle, Transform)>> {
        self.computed_components.as_ref()
    }
}

pub(crate) fn finish_component_dependency(
    component_files: &mut Vec<AssetComponent>,
    dependency_id: AssetId,
    handle: AssetComponentHandle,
) {
    let mut found = false;
    for file in component_files.iter_mut() {
        match file {
            AssetComponent::Skin(AssetDependency::<SkinData>::AssetId(asset_id)) => {
                if asset_id == &dependency_id {
                    let AssetComponentHandle::Skin(handle) = handle else {
                        panic!("expected skin handle");
                    };
                    *file = AssetComponent::Skin(AssetDependency::<SkinData>::Handle(handle));
                    found = true;
                    break;
                }
            }
            AssetComponent::Scene(AssetDependency::<SceneData>::AssetId(asset_id)) => {
                if asset_id == &dependency_id {
                    let AssetComponentHandle::Scene(handle) = handle else {
                        panic!("expected scene handle");
                    };
                    *file = AssetComponent::Scene(AssetDependency::<SceneData>::Handle(handle));
                    found = true;
                    break;
                }
            }
            _ => {}
        }
    }
    if !found {
        panic!("unable to find dependency path for: {:?}", &dependency_id);
    }
}

impl From<String> for SceneData {
    fn from(path: String) -> Self {
        let file_path = format!("assets/{}", path);

        let Ok(data) = web_fs::read(&file_path) else {
            panic!("unable to read file: {:?}", &file_path);
        };

        let actions = asset_io::bits::SceneAction::read(&data).expect("unable to parse file");

        info!("--- reading scene: {} ---", path);

        let mut component_files = Vec::new();
        let mut net_transforms = Vec::new();
        let mut file_index = 0;
        for action in actions {
            match action {
                asset_io::bits::SceneAction::Component(asset_id, file_type) => {
                    info!(
                        "SkinOrSceneFile {} - type: {:?}, asset_id: {:?}. ",
                        file_index, file_type, asset_id
                    );

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
                asset_io::bits::SceneAction::NetTransform(
                    file_index,
                    x,
                    y,
                    z,
                    scale_x,
                    scale_y,
                    scale_z,
                    rotation,
                ) => {
                    info!("NetTransform {} : file_index: {}, position ({} {} {}), scale: ({} {} {}), rotation: ({}, {}, {}, {})",
                             net_transforms.len(),
                             file_index,
                             x, y, z,
                             scale_x, scale_y, scale_z,
                             rotation.x, rotation.y, rotation.z, rotation.w);
                    let transform =
                        Transform::from_translation(Vec3::new(x as f32, y as f32, z as f32))
                            .with_scale(Vec3::new(scale_x, scale_y, scale_z))
                            .with_rotation(Quat::from_xyzw(
                                rotation.x, rotation.y, rotation.z, rotation.w,
                            ));
                    net_transforms.push((file_index as usize, transform));
                }
            }
        }

        info!("--- done reading scene ---");

        Self {
            component_files,
            net_transforms,
            computed_components: None,
        }
    }
}
