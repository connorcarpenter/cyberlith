use cfg_if::cfg_if;

use serde::{Deserialize, Serialize};

use crate::json::{MAX_QUAT_COMPONENT_SIZE, MAX_SCALE};

cfg_if! {
    if #[cfg(feature = "read")] {
        mod read;
        pub use read::*;
    } else {}
}

cfg_if! {
    if #[cfg(feature = "write")] {
        mod write;
        pub use write::*;
    } else {}
}

// Scene

#[derive(Serialize, Deserialize, Clone)]
pub struct SceneFileComponent {
    asset_id: String,
    kind: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SceneFileTransform {
    component_id: u16,
    position: SceneFileTransformPosition,
    rotation: SceneFileTransformRotation,
    scale: SceneFileTransformScale,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SceneFileTransformPosition {
    x: i16, y: i16, z: i16,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SceneFileTransformRotation {
    x: i8, y: i8, z: i8, w: i8,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SceneFileTransformScale {
    x: u32, y: u32, z: u32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SceneFile {
    components: Vec<SceneFileComponent>,
    transforms: Vec<SceneFileTransform>,
}

impl SceneFile {
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
        self.transforms.push(SceneFileTransform {
            component_id,
            position: SceneFileTransformPosition {
                x,
                y,
                z,
            },
            rotation: SceneFileTransformRotation {
                x: (rotation_x * MAX_QUAT_COMPONENT_SIZE).round() as i8,
                y: (rotation_y * MAX_QUAT_COMPONENT_SIZE).round() as i8,
                z: (rotation_z * MAX_QUAT_COMPONENT_SIZE).round() as i8,
                w: (rotation_w * MAX_QUAT_COMPONENT_SIZE).round() as i8,
            },
            scale: SceneFileTransformScale {
                x: (scale_x * MAX_SCALE) as u32,
                y: (scale_y * MAX_SCALE) as u32,
                z: (scale_z * MAX_SCALE) as u32,
            },
        });
    }
}

impl SceneFile {
    pub fn new() -> Self {
        Self {
            components: Vec::new(),
            transforms: Vec::new(),
        }
    }

    pub fn add_component(&mut self, asset_id: &str, kind: &str) {
        self.components.push(SceneFileComponent {
            asset_id: asset_id.to_string(),
            kind: kind.to_string(),
        });
    }
}