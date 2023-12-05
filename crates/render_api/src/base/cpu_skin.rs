use std::hash::Hash;

use crate::{AssetHash, base::CpuMaterial, Handle};

#[derive(Debug, Clone)]
pub struct CpuSkin {
    // index in this Vec is FaceId of mesh
    face_to_material: Vec<Handle<CpuMaterial>>,
}

impl Default for CpuSkin {
    fn default() -> Self {
        Self {
            face_to_material: Vec::new(),
        }
    }
}

impl CpuSkin {
    pub fn add_face_color(&mut self, material: Handle<CpuMaterial>) {
        self.face_to_material.push(material);
    }

    pub fn len(&self) -> usize {
        self.face_to_material.len()
    }

    pub fn face_to_material_list(&self) -> &Vec<Handle<CpuMaterial>> {
        &self.face_to_material
    }
}