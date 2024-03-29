use asset_id::AssetId;
use serde::{Deserialize, Serialize};

use crate::json::{MAX_QUAT_COMPONENT_SIZE, MAX_SCALE};

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum FileComponentType {
    Skin,
    Scene,
}

impl FileComponentType {
    pub fn to_string(&self) -> String {
        match self {
            FileComponentType::Skin => "skin".to_string(),
            FileComponentType::Scene => "scene".to_string(),
        }
    }

    pub fn from_string(kind: &str) -> Self {
        match kind {
            "skin" => FileComponentType::Skin,
            "scene" => FileComponentType::Scene,
            _ => panic!("Unknown kind: {}", kind),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct FileComponentEntry {
    asset_id: String,
    kind: String,
}

impl FileComponentEntry {
    pub fn new(asset_id: AssetId, kind: FileComponentType) -> Self {
        Self {
            asset_id: asset_id.as_string(),
            kind: kind.to_string(),
        }
    }

    pub fn asset_id(&self) -> AssetId {
        AssetId::from_str(self.asset_id.as_str()).unwrap()
    }

    pub fn kind(&self) -> FileComponentType {
        FileComponentType::from_string(&self.kind)
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct FileTransformPosition {
    x: i16,
    y: i16,
    z: i16,
}

impl FileTransformPosition {
    pub fn new(x: i16, y: i16, z: i16) -> Self {
        Self { x, y, z }
    }

    pub fn x(&self) -> i16 {
        self.x
    }

    pub fn y(&self) -> i16 {
        self.y
    }

    pub fn z(&self) -> i16 {
        self.z
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct FileTransformRotation {
    x: i8,
    y: i8,
    z: i8,
    w: i8,
}

impl FileTransformRotation {
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self {
            x: (x * MAX_QUAT_COMPONENT_SIZE).round() as i8,
            y: (y * MAX_QUAT_COMPONENT_SIZE).round() as i8,
            z: (z * MAX_QUAT_COMPONENT_SIZE).round() as i8,
            w: (w * MAX_QUAT_COMPONENT_SIZE).round() as i8,
        }
    }

    pub fn x(&self) -> f32 {
        self.x as f32 / MAX_QUAT_COMPONENT_SIZE
    }

    pub fn y(&self) -> f32 {
        self.y as f32 / MAX_QUAT_COMPONENT_SIZE
    }

    pub fn z(&self) -> f32 {
        self.z as f32 / MAX_QUAT_COMPONENT_SIZE
    }

    pub fn w(&self) -> f32 {
        self.w as f32 / MAX_QUAT_COMPONENT_SIZE
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct FileTransformScale {
    x: u32,
    y: u32,
    z: u32,
}

impl FileTransformScale {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self {
            x: (x * MAX_SCALE) as u32,
            y: (y * MAX_SCALE) as u32,
            z: (z * MAX_SCALE) as u32,
        }
    }

    pub fn x(&self) -> f32 {
        self.x as f32 / MAX_SCALE
    }

    pub fn y(&self) -> f32 {
        self.y as f32 / MAX_SCALE
    }

    pub fn z(&self) -> f32 {
        self.z as f32 / MAX_SCALE
    }
}
