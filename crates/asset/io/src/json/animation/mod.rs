use cfg_if::cfg_if;

use serde::{Deserialize, Serialize};

use crate::json::MAX_QUAT_COMPONENT_SIZE;

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

// Animation

#[derive(Serialize, Deserialize, Clone)]
pub struct AnimFileQuat {
    x: i8,
    y: i8,
    z: i8,
    w: i8,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AnimFilePose {
    edge_id: u16,
    rotation: AnimFileQuat,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AnimFileFrame {
    poses: Vec<AnimFilePose>,
    transition_ms: u16,
}

impl AnimFileFrame {
    pub fn new(transition_ms: u16) -> Self {
        Self {
            poses: Vec::new(),
            transition_ms,
        }
    }

    pub fn add_pose(&mut self, edge_id: u16, rotation_x: f32, rotation_y: f32, rotation_z: f32, rotation_w: f32) {
        self.poses.push(AnimFilePose {
            edge_id,
            rotation: AnimFileQuat {
                x: (rotation_x * MAX_QUAT_COMPONENT_SIZE).round() as i8,
                y: (rotation_y * MAX_QUAT_COMPONENT_SIZE).round() as i8,
                z: (rotation_z * MAX_QUAT_COMPONENT_SIZE).round() as i8,
                w: (rotation_w * MAX_QUAT_COMPONENT_SIZE).round() as i8,
            },
        });
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AnimFile {
    skeleton_asset_id: String,
    edge_names: Vec<String>,
    frames: Vec<AnimFileFrame>,
}

impl AnimFile {
    pub fn new() -> Self {
        Self {
            skeleton_asset_id: String::new(),
            edge_names: Vec::new(),
            frames: Vec::new(),
        }
    }

    pub fn set_skeleton_asset_id(&mut self, asset_id: &str) {
        self.skeleton_asset_id = asset_id.to_string();
    }

    pub fn add_edge_name(&mut self, name: &str) {
        self.edge_names.push(name.to_string());
    }

    pub fn add_frame(&mut self, frame: AnimFileFrame) {
        self.frames.push(frame);
    }
}