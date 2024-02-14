use std::{ops::{Deref, DerefMut}, marker::PhantomData};

use bevy_ecs::prelude::Component;

use naia_bevy_shared::{BitReader, BitWrite, EntityProperty, Property, Protocol, ProtocolPlugin, Replicate, Serde, SerdeErr};

use crypto::U32Token;

// Plugin
pub(crate) struct AssetRefsPlugin;

impl ProtocolPlugin for AssetRefsPlugin {
    fn build(&self, protocol: &mut Protocol) {
        protocol
            .add_component::<AssetRef<Body>>();
    }
}

// Asset Markers
pub struct Body;

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

// AssetId
#[derive(Clone, Copy, PartialEq, Hash, Eq, Debug)]
pub struct AssetId(U32Token);

impl Serde for AssetId {
    fn ser(&self, writer: &mut dyn BitWrite) {
        self.0.as_u32().ser(writer);
    }

    fn de(reader: &mut BitReader) -> Result<Self, SerdeErr> {
        let val = u32::de(reader)?;
        let Some(val) = U32Token::from_u32(val) else {
            return Err(SerdeErr);
        } ;
        Ok(Self(val))
    }

    fn bit_length(&self) -> u32 {
        self.0.as_u32().bit_length()
    }
}

impl AssetId {
    pub fn from_str(s: &str) -> Result<Self, SerdeErr> {
        let Some(val) = U32Token::from_str(s) else {
            return Err(SerdeErr);
        };
        Ok(Self(val))
    }

    pub fn from_u32(n: u32) -> Result<Self, SerdeErr> {
        let Some(val) = U32Token::from_u32(n) else {
            return Err(SerdeErr);
        };
        Ok(Self(val))
    }

    pub fn to_string(&self) -> String {
        self.0.as_string()
    }

    pub fn to_u32(&self) -> u32 {
        self.0.as_u32()
    }
}

impl Deref for AssetId {
    type Target = U32Token;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for AssetId {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}