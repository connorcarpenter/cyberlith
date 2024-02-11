use cfg_if::cfg_if;

use serde::{Deserialize, Serialize};
use crypto::U32Token;

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

impl AnimFileQuat {

    pub fn from_xyzw(rotation_x: f32, rotation_y: f32, rotation_z: f32, rotation_w: f32) -> Self {
        Self {
            x: (rotation_x * MAX_QUAT_COMPONENT_SIZE).round() as i8,
            y: (rotation_y * MAX_QUAT_COMPONENT_SIZE).round() as i8,
            z: (rotation_z * MAX_QUAT_COMPONENT_SIZE).round() as i8,
            w: (rotation_w * MAX_QUAT_COMPONENT_SIZE).round() as i8,
        }
    }

    pub fn get_x(&self) -> f32 {
        self.x as f32 / MAX_QUAT_COMPONENT_SIZE
    }

    pub fn get_y(&self) -> f32 {
        self.y as f32 / MAX_QUAT_COMPONENT_SIZE
    }

    pub fn get_z(&self) -> f32 {
        self.z as f32 / MAX_QUAT_COMPONENT_SIZE
    }

    pub fn get_w(&self) -> f32 {
        self.w as f32 / MAX_QUAT_COMPONENT_SIZE
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AnimFilePose {
    edge_id: u16,
    rotation: AnimFileQuat,
}

impl AnimFilePose {
    pub fn get_edge_id(&self) -> u16 {
        self.edge_id
    }

    pub fn get_rotation(&self) -> &AnimFileQuat {
        &self.rotation
    }
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
            rotation: AnimFileQuat::from_xyzw(rotation_x, rotation_y, rotation_z, rotation_w),
        });
    }

    pub fn get_poses(&self) -> &Vec<AnimFilePose> {
        &self.poses
    }

    pub fn get_transition_ms(&self) -> u16 {
        self.transition_ms
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AnimFile {
    skeleton_asset_id: String,
    edge_names: Vec<String>,
    frames: Vec<AnimFileFrame>,
}

impl AnimFile {

    pub const CURRENT_SCHEMA_VERSION: u32 = 1;

    pub fn new() -> Self {
        Self {
            skeleton_asset_id: String::new(),
            edge_names: Vec::new(),
            frames: Vec::new(),
        }
    }

    pub fn get_skeleton_asset_id(&self) -> U32Token {
        U32Token::from_str(&self.skeleton_asset_id).unwrap()
    }

    pub fn set_skeleton_asset_id(&mut self, asset_id: &U32Token) {
        self.skeleton_asset_id = asset_id.as_string();
    }

    pub fn add_edge_name(&mut self, name: &str) {
        self.edge_names.push(name.to_string());
    }

    pub fn get_edge_names(&self) -> &Vec<String> {
        &self.edge_names
    }

    pub fn add_frame(&mut self, frame: AnimFileFrame) {
        self.frames.push(frame);
    }

    pub fn get_frames(&self) -> &Vec<AnimFileFrame> {
        &self.frames
    }
}