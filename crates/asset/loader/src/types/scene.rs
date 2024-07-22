
use asset_serde::bits::ComponentFileType;
use math::{Quat, Vec3};
use render_api::components::Transform;

use crate::{
    asset_dependency::{AssetComponent, AssetComponentHandle, AssetDependency},
    AssetHandle, SkinData, TypedAssetId,
};

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
        handle: AssetHandle<Self>,
        dependencies: &mut Vec<(TypedAssetId, TypedAssetId)>,
    ) {
        for file in self.component_files.iter() {
            match file {
                AssetComponent::Skin(AssetDependency::<SkinData>::AssetId(asset_id)) => {
                    dependencies.push((handle.into(), TypedAssetId::Skin(asset_id.clone())));
                }
                AssetComponent::Scene(AssetDependency::<SceneData>::AssetId(path)) => {
                    dependencies.push((handle.into(), TypedAssetId::Scene(path.clone())));
                }
                _ => {
                    panic!("expected unloaded (no handles!) skin or scene file");
                }
            }
        }
    }

    pub(crate) fn finish_dependency(&mut self, dependency_typed_id: TypedAssetId) {
        match dependency_typed_id {
            TypedAssetId::Skin(asset_id) => {
                // info!("finished scene dependency for skin: {:?}", asset_id,);
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

        if self.all_dependencies_loaded() {
            // compute components
            let mut components = Vec::new();
            for (file_index, transform) in self.net_transforms.iter() {
                let file = &self.component_files[*file_index];
                match file {
                    AssetComponent::Skin(AssetDependency::<SkinData>::AssetHandle(handle)) => {
                        components.push((AssetComponentHandle::Skin(*handle), *transform));
                    }
                    AssetComponent::Scene(AssetDependency::<SceneData>::AssetHandle(handle)) => {
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
                AssetComponent::Skin(AssetDependency::<SkinData>::AssetHandle(_)) => {}
                AssetComponent::Scene(AssetDependency::<SceneData>::AssetHandle(_)) => {}
                _ => {
                    return false;
                }
            }
        }
        true
    }

    pub fn get_components(&self) -> Option<&Vec<(AssetComponentHandle, Transform)>> {
        self.computed_components.as_ref()
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        let actions = asset_serde::bits::SceneAction::read(bytes).expect("unable to parse file");

        // info!("--- reading scene ---");

        let mut component_files = Vec::new();
        let mut net_transforms = Vec::new();
        // let mut file_index = 0;

        for action in actions {
            match action {
                asset_serde::bits::SceneAction::Component(asset_id, file_type) => {
                    // info!(
                    //     "SkinOrSceneFile {} - type: {:?}, asset_id: {:?}. ",
                    //     file_index, file_type, asset_id
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

                    // file_index += 1;
                }
                asset_serde::bits::SceneAction::NetTransform(
                    file_index,
                    x,
                    y,
                    z,
                    scale_x,
                    scale_y,
                    scale_z,
                    rotation,
                ) => {
                    // info!("NetTransform {} : file_index: {}, position ({} {} {}), scale: ({} {} {}), rotation: ({}, {}, {}, {})",
                    //          net_transforms.len(),
                    //          file_index,
                    //          x, y, z,
                    //          scale_x, scale_y, scale_z,
                    //          rotation.x, rotation.y, rotation.z, rotation.w);
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

        // info!("--- done reading scene ---");

        Self {
            component_files,
            net_transforms,
            computed_components: None,
        }
    }
}

pub(crate) fn finish_component_dependency(
    component_files: &mut Vec<AssetComponent>,
    component_handle: AssetComponentHandle,
) {
    let mut found = false;
    for file in component_files.iter_mut() {
        let dependency_id = component_handle.asset_id();
        match file {
            AssetComponent::Skin(AssetDependency::<SkinData>::AssetId(asset_id)) => {
                if asset_id == &dependency_id {
                    let AssetComponentHandle::Skin(asset_handle) = component_handle else {
                        panic!("expected skin handle");
                    };
                    *file = AssetComponent::Skin(AssetDependency::<SkinData>::AssetHandle(
                        asset_handle,
                    ));
                    found = true;
                    break;
                }
            }
            AssetComponent::Scene(AssetDependency::<SceneData>::AssetId(asset_id)) => {
                if asset_id == &dependency_id {
                    let AssetComponentHandle::Scene(handle) = component_handle else {
                        panic!("expected scene handle");
                    };
                    *file =
                        AssetComponent::Scene(AssetDependency::<SceneData>::AssetHandle(handle));
                    found = true;
                    break;
                }
            }
            _ => {}
        }
    }
    if !found {
        panic!(
            "unable to find dependency path for: {:?}",
            component_handle.asset_id()
        );
    }
}
