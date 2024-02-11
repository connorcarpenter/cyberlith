use cfg_if::cfg_if;

use serde::{Deserialize, Serialize};

use crate::json::{AssetId, MAX_QUAT_COMPONENT_SIZE, MAX_SCALE};

cfg_if! {
    if #[cfg(feature = "read_json")] {
        mod read;
        pub use read::*;
    } else {}
}

cfg_if! {
    if #[cfg(feature = "write_json")] {
        mod write;
        pub use write::*;
    } else {}
}

// Scene

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum SceneFileComponentType {
    Skin,
    Scene,
}

impl SceneFileComponentType {
    pub fn to_string(&self) -> String {
        match self {
            SceneFileComponentType::Skin => "skin".to_string(),
            SceneFileComponentType::Scene => "scene".to_string(),
        }
    }

    pub fn from_string(kind: &str) -> Self {
        match kind {
            "skin" => SceneFileComponentType::Skin,
            "scene" => SceneFileComponentType::Scene,
            _ => panic!("Unknown kind: {}", kind),
        }
    }
}


#[derive(Serialize, Deserialize, Clone)]
pub struct SceneFileComponent {
    asset_id: String,
    kind: String,
}

impl SceneFileComponent {

    pub fn new(asset_id: AssetId, kind: SceneFileComponentType) -> Self {
        Self {
            asset_id: asset_id.as_string(),
            kind: kind.to_string(),
        }
    }

    pub fn asset_id(&self) -> AssetId {
        AssetId::from_str(self.asset_id.as_str()).unwrap()
    }

    pub fn kind(&self) -> SceneFileComponentType {
        SceneFileComponentType::from_string(&self.kind)
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SceneFileTransform {
    component_id: u16,
    position: SceneFileTransformPosition,
    rotation: SceneFileTransformRotation,
    scale: SceneFileTransformScale,
}

impl SceneFileTransform {

    pub fn new(component_id: u16, position: SceneFileTransformPosition, rotation: SceneFileTransformRotation, scale: SceneFileTransformScale) -> Self {
        Self {
            component_id,
            position,
            rotation,
            scale,
        }
    }

    pub fn component_id(&self) -> u16 {
        self.component_id
    }

    pub fn position(&self) -> SceneFileTransformPosition {
        self.position.clone()
    }

    pub fn rotation(&self) -> SceneFileTransformRotation {
        self.rotation.clone()
    }

    pub fn scale(&self) -> SceneFileTransformScale {
        self.scale.clone()
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SceneFileTransformPosition {
    x: i16, y: i16, z: i16,
}

impl SceneFileTransformPosition {
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
pub struct SceneFileTransformRotation {
    x: i8, y: i8, z: i8, w: i8,
}

impl SceneFileTransformRotation {
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
pub struct SceneFileTransformScale {
    x: u32, y: u32, z: u32,
}

impl SceneFileTransformScale {

    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self {
            x: (x * MAX_SCALE) as u32,
            y: (y * MAX_SCALE) as u32,
            z: (z * MAX_SCALE) as u32
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

#[derive(Serialize, Deserialize, Clone)]
pub struct SceneFile {
    components: Vec<SceneFileComponent>,
    transforms: Vec<SceneFileTransform>,
}

impl SceneFile {

    pub const CURRENT_SCHEMA_VERSION: u32 = 1;

    pub fn new() -> Self {
        Self {
            components: Vec::new(),
            transforms: Vec::new(),
        }
    }

    pub fn get_components(&self) -> &Vec<SceneFileComponent> {
        &self.components
    }

    pub fn add_component(&mut self, asset_id: AssetId, kind: SceneFileComponentType) {
        self.components.push(SceneFileComponent::new(asset_id, kind));
    }

    pub fn get_transforms(&self) -> &Vec<SceneFileTransform> {
        &self.transforms
    }

    pub fn add_transform(
        &mut self,
        component_id: u16,
        x: i16,
        y: i16,
        z: i16,
        scale_x: f32,
        scale_y: f32,
        scale_z: f32,
        rotation_x: f32,
        rotation_y: f32,
        rotation_z: f32,
        rotation_w: f32
    ) {
        self.transforms.push(SceneFileTransform::new(
            component_id,
            SceneFileTransformPosition::new(x, y, z),
            SceneFileTransformRotation::new(rotation_x, rotation_y, rotation_z, rotation_w),
            SceneFileTransformScale::new(scale_x, scale_y, scale_z)
        ));
    }
}