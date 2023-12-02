
use render_api::Handle;

use crate::{AnimationData, IconData, MeshFile, ModelData, PaletteData, SceneData, SkeletonData, SkinData};

pub struct AssetHandle {
    inner: AssetHandleImpl,
}

impl AssetHandle {
    pub(crate) fn to_impl(self) -> AssetHandleImpl {
        self.inner
    }
}

pub(crate) enum AssetHandleImpl {
    Mesh(Handle<MeshFile>),
    Skeleton(Handle<SkeletonData>),
    Palette(Handle<PaletteData>),
    Animation(Handle<AnimationData>),
    Icon(Handle<IconData>),
    Skin(Handle<SkinData>),
    Model(Handle<ModelData>),
    Scene(Handle<SceneData>),
}

impl From<Handle<MeshFile>> for AssetHandle {
    fn from(handle: Handle<MeshFile>) -> Self {
        Self {
            inner: AssetHandleImpl::Mesh(handle),
        }
    }
}

impl From<Handle<SkeletonData>> for AssetHandle {
    fn from(handle: Handle<SkeletonData>) -> Self {
        Self {
            inner: AssetHandleImpl::Skeleton(handle),
        }
    }
}

impl From<Handle<PaletteData>> for AssetHandle {
    fn from(handle: Handle<PaletteData>) -> Self {
        Self {
            inner: AssetHandleImpl::Palette(handle),
        }
    }
}

impl From<Handle<AnimationData>> for AssetHandle {
    fn from(handle: Handle<AnimationData>) -> Self {
        Self {
            inner: AssetHandleImpl::Animation(handle),
        }
    }
}

impl From<Handle<IconData>> for AssetHandle {
    fn from(handle: Handle<IconData>) -> Self {
        Self {
            inner: AssetHandleImpl::Icon(handle),
        }
    }
}

impl From<Handle<SkinData>> for AssetHandle {
    fn from(handle: Handle<SkinData>) -> Self {
        Self {
            inner: AssetHandleImpl::Skin(handle),
        }
    }
}

impl From<Handle<ModelData>> for AssetHandle {
    fn from(handle: Handle<ModelData>) -> Self {
        Self {
            inner: AssetHandleImpl::Model(handle),
        }
    }
}

impl From<Handle<SceneData>> for AssetHandle {
    fn from(handle: Handle<SceneData>) -> Self {
        Self {
            inner: AssetHandleImpl::Scene(handle),
        }
    }
}



