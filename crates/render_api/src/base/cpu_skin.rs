use std::hash::Hash;

use bevy_log::info;

use crate::{AssetHash, base::CpuMaterial, Handle};

#[derive(Clone)]
pub struct CpuSkin {
    // index in this Vec is FaceId of mesh
    face_to_material: Vec<Handle<CpuMaterial>>,
}

impl CpuSkin {
    pub fn log(&self) {
        info!("--- loaded cpu skin ---");
        for (index, handle) in self.face_to_material.iter().enumerate() {
            info!("face: {}, material: {:?}", index, handle.id);
        }
        info!("--- end cpu skin ---");
    }
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