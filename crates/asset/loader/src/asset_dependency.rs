use asset_id::AssetId;
use storage::Handle;

use crate::{AssetHandle, SceneData, SkinData};

pub(crate) enum AssetDependency<T> {
    AssetId(AssetId),
    Bytes(Box<[u8]>),
    AssetHandle(AssetHandle<T>),
    Handle(Handle<T>),
}

impl<T> AssetDependency<T> {

    pub(crate) fn get_asset_id(&self) -> AssetId {
        match self {
            AssetDependency::AssetId(asset_id) => *asset_id,
            _ => panic!("expected asset id"),
        }
    }

    pub(crate) fn load_handle(&mut self, handle: Handle<T>) {
        if let AssetDependency::Handle(_) = self {
            panic!("cannot load handle twice!");
        }

        *self = AssetDependency::Handle(handle);
    }

    pub(crate) fn load_asset_handle(&mut self, asset_handle: AssetHandle<T>) {
        if let AssetDependency::AssetHandle(_) = self {
            panic!("cannot load asset handle twice!");
        }

        *self = AssetDependency::AssetHandle(asset_handle);
    }
}

pub(crate) enum AssetComponent {
    Skin(AssetDependency<SkinData>),
    Scene(AssetDependency<SceneData>),
}

pub enum AssetComponentHandle {
    Skin(AssetHandle<SkinData>),
    Scene(AssetHandle<SceneData>),
}

impl AssetComponentHandle {
    pub fn asset_id(&self) -> AssetId {
        match self {
            AssetComponentHandle::Skin(handle) => handle.asset_id(),
            AssetComponentHandle::Scene(handle) => handle.asset_id(),
        }
    }
}
