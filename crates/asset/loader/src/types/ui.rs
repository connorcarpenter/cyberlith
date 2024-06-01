
use crate::TypedAssetId;

pub struct UiDependencies {}

impl Default for UiDependencies {
    fn default() -> Self {
        panic!("");
    }
}

impl UiDependencies {
    pub fn new() -> Self {
        Self {}
    }

    pub fn load_dependencies(
        &self,
        _asset_handle: TypedAssetId,
        _dependencies: &mut Vec<(TypedAssetId, TypedAssetId)>,
    ) {
        // {
        //     let AssetDependency::<IconData>::AssetId(asset_id) = &self.text_icon else {
        //         panic!("expected path right after load");
        //     };
        //     dependencies.push((asset_handle, TypedAssetId::Icon(*asset_id)));
        // }
        //
        // {
        //     let AssetDependency::<IconData>::AssetId(asset_id) = &self.eye_icon else {
        //         panic!("expected path right after load");
        //     };
        //     dependencies.push((asset_handle, TypedAssetId::Icon(*asset_id)));
        // }
    }

    pub fn finish_dependency(&mut self, dependency_typed_id: TypedAssetId) {
        match dependency_typed_id {
            TypedAssetId::Icon(id) => match id.as_string().as_str() {
                // "34mvvk" => {
                //     self.text_icon = AssetDependency::AssetHandle(AssetHandle::<IconData>::new(id));
                // }
                // "qbgz5j" => {
                //     self.eye_icon = AssetDependency::AssetHandle(AssetHandle::<IconData>::new(id));
                // }
                _ => {
                    panic!("unexpected icon id");
                }
            },
            _ => {
                panic!("unexpected type of handle");
            }
        }
    }
}
