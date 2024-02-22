use std::marker::PhantomData;

use bevy_ecs::prelude::Component;

use naia_bevy_shared::{EntityProperty, Property, Protocol, ProtocolPlugin, Replicate};

use asset_id::AssetId;

// Plugin
pub(crate) struct AssetRefsPlugin;

impl ProtocolPlugin for AssetRefsPlugin {
    fn build(&self, protocol: &mut Protocol) {
        protocol.add_component::<AssetRef<Main>>().add_component::<AssetEntry>();
    }
}

// Asset Markers
pub struct Main;

// AssetRef
#[derive(Component, Replicate)]
pub struct AssetRef<T: Send + Sync + 'static> {
    pub asset_id_entity: EntityProperty,
    _marker: PhantomData<T>,
}

impl<T: Send + Sync + 'static> AssetRef<T> {
    pub fn new() -> Self {
        Self::new_complete(PhantomData)
    }
}

// AssetEntry
#[derive(Component, Replicate)]
pub struct AssetEntry {
    pub asset_id: Property<AssetId>,
}

impl AssetEntry {
    pub fn new(asset_id: AssetId) -> Self {
        Self::new_complete(asset_id)
    }
}
