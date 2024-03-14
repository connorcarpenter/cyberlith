
use crate::{asset_dependency::AssetDependency, AssetHandle, IconData, TypedAssetId};

pub struct UiData {
    icon_file: AssetDependency<IconData>,
}

impl Default for UiData {
    fn default() -> Self {
        panic!("");
    }
}

impl UiData {
    pub(crate) fn load_dependencies(
        &self,
        asset_handle: AssetHandle<Self>,
        dependencies: &mut Vec<(TypedAssetId, TypedAssetId)>,
    ) {
        let AssetDependency::<IconData>::AssetId(asset_id) = &self.icon_file else {
            panic!("expected path right after load");
        };
        dependencies.push((asset_handle.into(), TypedAssetId::Icon(*asset_id)));
    }

    pub(crate) fn finish_dependency(&mut self, dependency_typed_id: TypedAssetId) {
        match dependency_typed_id {
            TypedAssetId::Icon(id) => {
                let handle = AssetHandle::<IconData>::new(id);
                self.icon_file.load_asset_handle(handle);
            }
            _ => {
                panic!("unexpected type of handle");
            }
        }
    }

    pub(crate) fn get_icon_handle(&self) -> AssetHandle<IconData> {
        if let AssetDependency::<IconData>::AssetHandle(handle) = &self.icon_file {
            *handle
        } else {
            panic!("expected icon handle");
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        todo!()
    }
}
