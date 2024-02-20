use cfg_if::cfg_if;

use serde::{Deserialize, Serialize};
use asset_id::AssetId;

use crate::json::components::{FileComponentEntry, FileComponentType, FileTransformPosition, FileTransformRotation, FileTransformScale};

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

// Model

#[derive(Serialize, Deserialize, Clone)]
pub struct ModelFileTransform {
    component_id: u16,
    name: String,
    position: FileTransformPosition,
    rotation: FileTransformRotation,
    scale: FileTransformScale,
}

impl ModelFileTransform {

    pub fn new(component_id: u16, name: &str, position: FileTransformPosition, rotation: FileTransformRotation, scale: FileTransformScale) -> Self {
        Self {
            component_id,
            name: name.to_string(),
            position,
            rotation,
            scale,
        }
    }

    pub fn component_id(&self) -> u16 {
        self.component_id
    }

    pub fn name(&self) -> String {
        self.name.clone()
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
pub struct ModelFile {
    skeleton_id: String,
    components: Vec<FileComponentEntry>,
    transforms: Vec<ModelFileTransform>,
}

impl ModelFile {

    pub const CURRENT_SCHEMA_VERSION: u32 = 0;

    pub fn new() -> Self {
        Self {
            skeleton_id: String::new(),
            components: Vec::new(),
            transforms: Vec::new(),
        }
    }

    pub fn dependencies(&self) -> Vec<AssetId> {
        let mut output = Vec::new();

        output.push(self.get_skeleton_id());

        for component in &self.components {
            output.push(component.asset_id());
        }

        output
    }

    pub fn get_skeleton_id(&self) -> AssetId {
        AssetId::from_str(self.skeleton_id.as_str()).unwrap()
    }

    pub fn set_skeleton_id(&mut self, asset_id: AssetId) {
        self.skeleton_id = asset_id.as_string();
    }

    pub fn get_components(&self) -> &Vec<FileComponentEntry> {
        &self.components
    }

    pub fn add_component(&mut self, asset_id: AssetId, kind: FileComponentType) {
        self.components.push(FileComponentEntry::new(asset_id, kind));
    }

    pub fn get_transforms(&self) -> &Vec<ModelFileTransform> {
        &self.transforms
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
        self.transforms.push(ModelFileTransform::new(
            component_id,
            name,
            FileTransformPosition::new(x, y, z),
            FileTransformRotation::new(rotation_x, rotation_y, rotation_z, rotation_w),
            FileTransformScale::new(scale_x, scale_y, scale_z)
        ));
    }
}