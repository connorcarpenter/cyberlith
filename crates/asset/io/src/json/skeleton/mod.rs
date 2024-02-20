use cfg_if::cfg_if;

use asset_id::AssetId;
use serde::{Deserialize, Serialize};

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

// Skeleton

#[derive(Serialize, Deserialize, Clone)]
pub struct SkelFileVertexParent {
    id: u16,
    rotation: u8,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SkelFileVertex {
    x: i16,
    y: i16,
    z: i16,
    parent: Option<SkelFileVertexParent>,
    name: Option<String>,
}

impl SkelFileVertex {
    pub fn new(
        x: i16,
        y: i16,
        z: i16,
        parent_opt: Option<(u16, u8)>,
        name_opt: Option<String>,
    ) -> Self {
        let parent = parent_opt.map(|(parent_id, rotation)| SkelFileVertexParent {
            id: parent_id,
            rotation,
        });
        Self {
            x,
            y,
            z,
            parent,
            name: name_opt,
        }
    }

    pub fn deconstruct(&self) -> (i16, i16, i16, Option<(u16, u8)>, Option<String>) {
        let parent = self
            .parent
            .as_ref()
            .map(|parent| (parent.id, parent.rotation));
        (self.x, self.y, self.z, parent, self.name.clone())
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SkelFile {
    vertices: Vec<SkelFileVertex>,
}

impl SkelFile {
    pub const CURRENT_SCHEMA_VERSION: u32 = 0;

    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
        }
    }

    pub fn dependencies(&self) -> Vec<AssetId> {
        Vec::new()
    }

    pub fn get_vertices(&self) -> &Vec<SkelFileVertex> {
        &self.vertices
    }

    pub fn add_vertex(
        &mut self,
        x: i16,
        y: i16,
        z: i16,
        parent_id_opt: Option<(u16, u8)>,
        name_opt: Option<String>,
    ) {
        self.vertices
            .push(SkelFileVertex::new(x, y, z, parent_id_opt, name_opt));
    }
}
