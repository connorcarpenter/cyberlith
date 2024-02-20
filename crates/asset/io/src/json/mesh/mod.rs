use cfg_if::cfg_if;

use serde::{Deserialize, Serialize};
use asset_id::AssetId;

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

// Mesh

#[derive(Serialize, Deserialize, Clone)]
pub struct MeshFileVertex {
    x: i16,
    y: i16,
    z: i16,
}

impl MeshFileVertex {
    pub fn deconstruct(&self) -> (i16, i16, i16) {
        (self.x, self.y, self.z)
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MeshFileEdge {
    vertex_a: u16,
    vertex_b: u16,
}

impl MeshFileEdge {
    pub fn deconstruct(&self) -> (u16, u16) {
        (self.vertex_a, self.vertex_b)
    }
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

impl MeshFileFace {
    pub fn deconstruct(&self) -> (u16, u16, u16, u16, u16, u16, u16) {
        (self.face_id, self.vertex_a, self.vertex_b, self.vertex_c, self.edge_a, self.edge_b, self.edge_c)
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MeshFile {
    vertices: Vec<MeshFileVertex>,
    edges: Vec<MeshFileEdge>,
    faces: Vec<MeshFileFace>,
}

impl MeshFile {

    pub const CURRENT_SCHEMA_VERSION: u32 = 0;

    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            edges: Vec::new(),
            faces: Vec::new(),
        }
    }

    pub fn dependencies(&self) -> Vec<AssetId> {
        Vec::new()
    }

    pub fn get_vertices(&self) -> &Vec<MeshFileVertex> {
        &self.vertices
    }

    pub fn add_vertex(&mut self, x: i16, y: i16, z: i16) {
        self.vertices.push(MeshFileVertex {
            x,
            y,
            z,
        });
    }

    pub fn get_edges(&self) -> &Vec<MeshFileEdge> {
        &self.edges
    }

    pub fn add_edge(&mut self, vertex_a: u16, vertex_b: u16) {
        self.edges.push(MeshFileEdge {
            vertex_a,
            vertex_b,
        });
    }

    pub fn get_faces(&self) -> &Vec<MeshFileFace> {
        &self.faces
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