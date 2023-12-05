
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

// Handle<T> -> AssetHandle

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

// AssetHandle -> Handle<T>

impl From<AssetHandle> for Handle<MeshFile> {
    fn from(value: AssetHandle) -> Self {
        match value.inner {
            AssetHandleImpl::Mesh(handle) => handle,
            _ => panic!("unexpected handle"),
        }
    }
}

impl From<AssetHandle> for Handle<PaletteData> {
    fn from(value: AssetHandle) -> Self {
        match value.inner {
            AssetHandleImpl::Palette(handle) => handle,
            _ => panic!("unexpected handle"),
        }
    }
}

impl From<AssetHandle> for Handle<SkeletonData> {
    fn from(value: AssetHandle) -> Self {
        match value.inner {
            AssetHandleImpl::Skeleton(handle) => handle,
            _ => panic!("unexpected handle"),
        }
    }
}

impl From<AssetHandle> for Handle<AnimationData> {
    fn from(value: AssetHandle) -> Self {
        match value.inner {
            AssetHandleImpl::Animation(handle) => handle,
            _ => panic!("unexpected handle"),
        }
    }
}

impl From<AssetHandle> for Handle<IconData> {
    fn from(value: AssetHandle) -> Self {
        match value.inner {
            AssetHandleImpl::Icon(handle) => handle,
            _ => panic!("unexpected handle"),
        }
    }
}

impl From<AssetHandle> for Handle<SkinData> {
    fn from(value: AssetHandle) -> Self {
        match value.inner {
            AssetHandleImpl::Skin(handle) => handle,
            _ => panic!("unexpected handle"),
        }
    }
}

impl From<AssetHandle> for Handle<ModelData> {
    fn from(value: AssetHandle) -> Self {
        match value.inner {
            AssetHandleImpl::Model(handle) => handle,
            _ => panic!("unexpected handle"),
        }
    }
}

impl From<AssetHandle> for Handle<SceneData> {
    fn from(value: AssetHandle) -> Self {
        match value.inner {
            AssetHandleImpl::Scene(handle) => handle,
            _ => panic!("unexpected handle"),
        }
    }
}

