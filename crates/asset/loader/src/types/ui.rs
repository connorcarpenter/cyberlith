use asset_id::AssetId;

use crate::{asset_dependency::AssetDependency, AssetHandle, IconData, TypedAssetId};

pub struct UiDependencies {
    text_icon: AssetDependency<IconData>,
    eye_icon: AssetDependency<IconData>,
}

impl Default for UiDependencies {
    fn default() -> Self {
        panic!("");
    }
}

impl UiDependencies {
    pub fn new(
        text_icon_asset_id: &AssetId,
        eye_icon_asset_id: &AssetId,
    ) -> Self {
        let text_icon = AssetDependency::AssetId(*text_icon_asset_id);
        let eye_icon = AssetDependency::AssetId(*eye_icon_asset_id);

        Self { text_icon, eye_icon }
    }

    pub fn load_dependencies(
        &self,
        asset_handle: TypedAssetId,
        dependencies: &mut Vec<(TypedAssetId, TypedAssetId)>,
    ) {
        {
            let AssetDependency::<IconData>::AssetId(asset_id) = &self.text_icon else {
                panic!("expected path right after load");
            };
            dependencies.push((asset_handle, TypedAssetId::Icon(*asset_id)));
        }

        {
            let AssetDependency::<IconData>::AssetId(asset_id) = &self.eye_icon else {
                panic!("expected path right after load");
            };
            dependencies.push((asset_handle, TypedAssetId::Icon(*asset_id)));
        }
    }

    pub fn finish_dependency(&mut self, dependency_typed_id: TypedAssetId) {
        match dependency_typed_id {
            TypedAssetId::Icon(id) => {
                match id.as_string().as_str() {
                    "34mvvk" => {
                        self.text_icon = AssetDependency::AssetHandle(AssetHandle::<IconData>::new(id));
                    }
                    "qbgz5j" => {
                        self.eye_icon = AssetDependency::AssetHandle(AssetHandle::<IconData>::new(id));
                    }
                    _ => {
                        panic!("unexpected icon id");
                    }
                }
            }
            _ => {
                panic!("unexpected type of handle");
            }
        }
    }

    pub fn get_text_icon_handle(&self) -> AssetHandle<IconData> {
        if let AssetDependency::<IconData>::AssetHandle(handle) = &self.text_icon {
            *handle
        } else {
            panic!("expected icon handle");
        }
    }

    pub fn get_eye_icon_handle(&self) -> AssetHandle<IconData> {
        if let AssetDependency::<IconData>::AssetHandle(handle) = &self.eye_icon {
            *handle
        } else {
            panic!("expected icon handle");
        }
    }
}
