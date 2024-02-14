use asset_io::AssetId;
use storage::Handle;

use crate::{SceneData, SkinData};

pub(crate) enum AssetDependency<T> {
    AssetId(AssetId),
    Handle(Handle<T>),
}

impl<T> AssetDependency<T> {
    pub(crate) fn load_handle(&mut self, handle: Handle<T>) {
        if let AssetDependency::AssetId(_) = self {
            *self = AssetDependency::Handle(handle);
        } else {
            panic!("cannot load handle twice!");
        }
    }
}

pub(crate) enum AssetComponent {
    Skin(AssetDependency<SkinData>),
    Scene(AssetDependency<SceneData>),
}

pub(crate) enum AssetComponentHandle {
    Skin(Handle<SkinData>),
    Scene(Handle<SceneData>),
}
