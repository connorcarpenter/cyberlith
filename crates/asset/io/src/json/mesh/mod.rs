use cfg_if::cfg_if;

use serde::{Deserialize, Serialize};

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

// Mesh

#[derive(Serialize, Deserialize, Clone)]
pub struct MeshFileVertex {
    x: i16,
    y: i16,
    z: i16,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MeshFileEdge {
    vertex_a: u16,
    vertex_b: u16,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MeshFileFace {
    face_id: u16,
    vertex_a: u16,
    vertex_b: u16,
    vertex_c: u16,
    edge_a: u16,
    edge_b: u16,
    edge_c: u16,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MeshFile {
    vertices: Vec<MeshFileVertex>,
    edges: Vec<MeshFileEdge>,
    faces: Vec<MeshFileFace>,
}

impl MeshFile {

    pub const CURRENT_SCHEMA_VERSION: u32 = 1;

    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            edges: Vec::new(),
            faces: Vec::new(),
        }
    }

    pub fn add_vertex(&mut self, x: i16, y: i16, z: i16) {
        self.vertices.push(MeshFileVertex {
            x,
            y,
            z,
        });
    }

    pub fn add_edge(&mut self, vertex_a: u16, vertex_b: u16) {
        self.edges.push(MeshFileEdge {
            vertex_a,
            vertex_b,
        });
    }

    pub fn add_face(&mut self, face_id: u16, vertex_a: u16, vertex_b: u16, vertex_c: u16, edge_a: u16, edge_b: u16, edge_c: u16) {
        self.faces.push(MeshFileFace {
            face_id,
            vertex_a,
            vertex_b,
            vertex_c,
            edge_a,
            edge_b,
            edge_c,
        });
    }
}