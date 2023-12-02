use render_api::Handle;

use crate::{SceneData, SkinData};

pub(crate) enum AssetDependency<T> {
    Path(String),
    Handle(Handle<T>),
}

pub(crate) enum SkinOrScene {
    Skin(AssetDependency<SkinData>),
    Scene(AssetDependency<SceneData>),
}