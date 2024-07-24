use std::collections::HashMap;

use asset_serde::bits::AnimatedModelBits;

use crate::{AnimationData, asset_dependency::AssetDependency, AssetHandle, ModelData, TypedAssetId};

pub struct AnimatedModelData {
    model_file: AssetDependency<ModelData>,
    animation_files: HashMap<String, AssetDependency<AnimationData>>,
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
        let mut animations = HashMap::new();

        for (animation_name, asset_id) in base.get_animations() {
            animations.insert(animation_name.clone(), AssetDependency::AssetId(*asset_id));
        }

        // info!("--- done reading animated model ---");

        Self {
            model_file: AssetDependency::AssetId(model_asset_id),
            animation_files: animations,
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

    pub(crate) fn load_dependencies(
        &self,
        handle: AssetHandle<Self>,
        dependencies: &mut Vec<(TypedAssetId, TypedAssetId)>,
    ) {
        let AssetDependency::<ModelData>::AssetId(asset_id) = &self.model_file else {
            panic!("expected path right after load");
        };
        dependencies.push((handle.into(), TypedAssetId::Model(asset_id.clone())));

        for (_animation_name, animation_dep) in &self.animation_files {
            let AssetDependency::<AnimationData>::AssetId(asset_id) = animation_dep else {
                panic!("expected path right after load");
            };
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
                for (_name, animation_file) in &mut self.animation_files {
                    if animation_file.get_asset_id() == asset_id {
                        animation_file.load_asset_handle(handle);
                        return;
                    }
                }
            }
            _ => {
                panic!("unexpected type of handle");
            }
        }
    }
}
