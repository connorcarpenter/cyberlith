use cfg_if::cfg_if;

use asset_id::AssetId;
use serde::{Deserialize, Serialize};

use crate::json::components::{
    FileComponentEntry, FileComponentType, FileTransformPosition, FileTransformRotation,
    FileTransformScale,
};

cfg_if! {
    if #[cfg(feature = "read_json")] {
        mod read;
    } else {}
}

cfg_if! {
    if #[cfg(feature = "write_json")] {
        mod write;
    } else {}
}

// Scene

#[derive(Serialize, Deserialize, Clone)]
pub struct SceneFileTransform {
    component_id: u16,
    position: FileTransformPosition,
    rotation: FileTransformRotation,
    scale: FileTransformScale,
}

impl SceneFileTransform {
    pub fn new(
        component_id: u16,
        position: FileTransformPosition,
        rotation: FileTransformRotation,
        scale: FileTransformScale,
    ) -> Self {
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

    pub fn position(&self) -> FileTransformPosition {
        self.position.clone()
    }

    pub fn rotation(&self) -> FileTransformRotation {
        self.rotation.clone()
    }

    pub fn scale(&self) -> FileTransformScale {
        self.scale.clone()
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SceneJson {
    components: Vec<FileComponentEntry>,
    transforms: Vec<SceneFileTransform>,
}

impl SceneJson {
    pub const CURRENT_SCHEMA_VERSION: u32 = 0;

    pub fn new() -> Self {
        Self {
            components: Vec::new(),
            transforms: Vec::new(),
        }
    }

    pub fn dependencies(&self) -> Vec<AssetId> {
        let mut output = Vec::new();

        for component in &self.components {
            output.push(component.asset_id());
        }

        output
    }

    pub fn get_components(&self) -> &Vec<FileComponentEntry> {
        &self.components
    }

    pub fn add_component(&mut self, asset_id: AssetId, kind: FileComponentType) {
        self.components
            .push(FileComponentEntry::new(asset_id, kind));
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
        rotation_w: f32,
    ) {
        self.transforms.push(SceneFileTransform::new(
            component_id,
            FileTransformPosition::new(x, y, z),
            FileTransformRotation::new(rotation_x, rotation_y, rotation_z, rotation_w),
            FileTransformScale::new(scale_x, scale_y, scale_z),
        ));
    }
}
