use storage::Handle;

use crate::{SceneData, SkinData};

pub(crate) enum AssetDependency<T> {
    Path(String),
    Handle(Handle<T>),
}

impl<T> AssetDependency<T> {
    pub(crate) fn load_handle(&mut self, handle: Handle<T>) {
        if let AssetDependency::Path(_) = self {
            *self = AssetDependency::Handle(handle);
        } else {
            panic!("cannot load handle twice!");
        }
    }
}

pub(crate) enum SkinOrScene {
    Skin(AssetDependency<SkinData>),
    Scene(AssetDependency<SceneData>),
}

pub(crate) enum SkinOrSceneHandle {
    Skin(Handle<SkinData>),
    Scene(Handle<SceneData>),
}
