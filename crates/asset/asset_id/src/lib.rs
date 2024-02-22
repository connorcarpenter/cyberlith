use std::ops::{Deref, DerefMut};

use naia_serde::{BitReader, BitWrite, SerdeInternal as Serde, SerdeErr};

use crypto::U32Token;

// AssetType
#[derive(Serde, Eq, PartialEq, Clone, Copy, Hash, Debug)]
pub enum AssetType {
    Mesh,
    Skeleton,
    Palette,
    Animation,
    Icon,
    Skin,
    Model,
    Scene,
}

impl AssetType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "mesh" => Some(Self::Mesh),
            "skeleton" => Some(Self::Skeleton),
            "palette" => Some(Self::Palette),
            "animation" => Some(Self::Animation),
            "icon" => Some(Self::Icon),
            "skin" => Some(Self::Skin),
            "model" => Some(Self::Model),
            "scene" => Some(Self::Scene),
            _ => None,
        }
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
        };
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

// ETag
#[derive(Clone, Copy, PartialEq, Hash, Eq, Debug)]
pub struct ETag(U32Token);

impl Serde for ETag {
    fn ser(&self, writer: &mut dyn BitWrite) {
        self.0.as_u32().ser(writer);
    }

    fn de(reader: &mut BitReader) -> Result<Self, SerdeErr> {
        let val = u32::de(reader)?;
        let Some(val) = U32Token::from_u32(val) else {
            return Err(SerdeErr);
        };
        Ok(Self(val))
    }

    fn bit_length(&self) -> u32 {
        self.0.as_u32().bit_length()
    }
}

impl ETag {
    pub fn new_random() -> Self {
        Self(U32Token::get_random())
    }

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

impl Deref for ETag {
    type Target = U32Token;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ETag {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
