use cfg_if::cfg_if;

use serde::{Deserialize, Serialize};

use crate::json::{MAX_QUAT_COMPONENT_SIZE, MAX_SCALE};

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

// Model

#[derive(Serialize, Deserialize, Clone)]
pub struct ModelFileComponent {
    asset_id: String,
    kind: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ModelFileTransform {
    component_id: u16,
    name: String,
    position: ModelFileTransformPosition,
    rotation: ModelFileTransformRotation,
    scale: ModelFileTransformScale,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ModelFileTransformPosition {
    x: i16, y: i16, z: i16,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ModelFileTransformRotation {
    x: i8, y: i8, z: i8, w: i8,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ModelFileTransformScale {
    x: u32, y: u32, z: u32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ModelFile {
    skeleton_id: String,
    components: Vec<ModelFileComponent>,
    transforms: Vec<ModelFileTransform>,
}

impl ModelFile {

    pub const CURRENT_SCHEMA_VERSION: u32 = 1;

    pub fn new() -> Self {
        Self {
            skeleton_id: String::new(),
            components: Vec::new(),
            transforms: Vec::new(),
        }
    }

    pub fn set_skeleton_id(&mut self, asset_id: &str) {
        self.skeleton_id = asset_id.to_string();
    }

    pub fn add_component(&mut self, asset_id: &str, kind: &str) {
        self.components.push(ModelFileComponent {
            asset_id: asset_id.to_string(),
            kind: kind.to_string(),
        });
    }

    pub fn add_transform(
        &mut self,
        component_id: u16,
        name: &str,
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
        self.transforms.push(ModelFileTransform {
            component_id,
            name: name.to_string(),
            position: ModelFileTransformPosition {
                x,
                y,
                z,
            },
            rotation: ModelFileTransformRotation {
                x: (rotation_x * MAX_QUAT_COMPONENT_SIZE).round() as i8,
                y: (rotation_y * MAX_QUAT_COMPONENT_SIZE).round() as i8,
                z: (rotation_z * MAX_QUAT_COMPONENT_SIZE).round() as i8,
                w: (rotation_w * MAX_QUAT_COMPONENT_SIZE).round() as i8,
            },
            scale: ModelFileTransformScale {
                x: (scale_x * MAX_SCALE) as u32,
                y: (scale_y * MAX_SCALE) as u32,
                z: (scale_z * MAX_SCALE) as u32,
            },
        });
    }
}