use asset_id::AssetId;
use asset_serde::bits::AnimatedModelBits;
use std::collections::HashMap;
// use logging::info;

use crate::{
    asset_dependency::AssetDependency, AnimationData, AssetHandle, ModelData, TypedAssetId,
};

pub struct AnimatedModelData {
    model_file: AssetDependency<ModelData>,
    animation_name_to_id: HashMap<String, AssetId>,
    animation_files: HashMap<AssetId, AssetDependency<AnimationData>>,
}

impl Default for AnimatedModelData {
    fn default() -> Self {
        panic!("");
    }
}

impl From<&[u8]> for AnimatedModelData {
    fn from(bytes: &[u8]) -> Self {
        // info!("--- reading animated model ---");

        let base = AnimatedModelBits::from_bytes(bytes).expect("unable to parse file");

        let model_asset_id = base.get_model_asset_id();
        let mut animation_name_to_id = HashMap::new();
        let mut animation_files = HashMap::new();

        for (animation_name, asset_id) in base.get_animations() {
            animation_name_to_id.insert(animation_name.clone(), *asset_id);
            animation_files.insert(*asset_id, AssetDependency::AssetId(*asset_id));
        }

        // info!("--- done reading animated model ---");

        Self {
            model_file: AssetDependency::AssetId(model_asset_id),
            animation_name_to_id,
            animation_files,
        }
    }
}

impl AnimatedModelData {
    pub fn get_model_file_handle(&self) -> Option<&AssetHandle<ModelData>> {
        if let AssetDependency::<ModelData>::AssetHandle(handle) = &self.model_file {
            Some(handle)
        } else {
            None
        }
    }

    pub fn get_animation_handle(&self, name: &str) -> Option<&AssetHandle<AnimationData>> {
        let asset_id = self.animation_name_to_id.get(name)?;
        let animation_file = self.animation_files.get(asset_id)?;
        let AssetDependency::<AnimationData>::AssetHandle(handle) = animation_file else {
            return None;
        };
        Some(handle)
    }

    pub(crate) fn load_dependencies(
        &self,
        handle: AssetHandle<Self>,
        dependencies: &mut Vec<(TypedAssetId, TypedAssetId)>,
    ) {
        // info!("loading dependencies for animated model: {:?}", handle.asset_id());
        let AssetDependency::<ModelData>::AssetId(asset_id) = &self.model_file else {
            panic!("expected path right after load");
        };
        // info!("  - loading model dependency: {:?}", asset_id);
        dependencies.push((handle.into(), TypedAssetId::Model(asset_id.clone())));

        for (_animation_name, animation_dep) in &self.animation_files {
            let AssetDependency::<AnimationData>::AssetId(asset_id) = animation_dep else {
                panic!("expected path right after load");
            };
            // info!("  - loading animation dependency: {:?}", asset_id);
            dependencies.push((handle.into(), TypedAssetId::Animation(asset_id.clone())));
        }
    }

    pub(crate) fn finish_dependency(&mut self, dependency_typed_id: TypedAssetId) {
        match dependency_typed_id {
            TypedAssetId::Model(asset_id) => {
                let handle = AssetHandle::<ModelData>::new(asset_id);
                self.model_file.load_asset_handle(handle);
            }
            TypedAssetId::Animation(asset_id) => {
                let handle = AssetHandle::<AnimationData>::new(asset_id);
                let Some(animation_file) = self.animation_files.get_mut(&asset_id) else {
                    panic!("unexpected animation file");
                };
                animation_file.load_asset_handle(handle);
            }
            _ => {
                panic!("unexpected type of handle");
            }
        }
    }
}
