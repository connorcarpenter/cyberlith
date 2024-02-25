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

// Mesh

#[derive(Serialize, Deserialize, Clone)]
pub struct MeshDataVertex {
    x: i16,
    y: i16,
    z: i16,
}

impl MeshDataVertex {
    pub fn deconstruct(&self) -> (i16, i16, i16) {
        (self.x, self.y, self.z)
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MeshDataEdge {
    vertex_a: u16,
    vertex_b: u16,
}

impl MeshDataEdge {
    pub fn deconstruct(&self) -> (u16, u16) {
        (self.vertex_a, self.vertex_b)
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MeshDataFace {
    face_id: u16,
    vertex_a: u16,
    vertex_b: u16,
    vertex_c: u16,
    edge_a: u16,
    edge_b: u16,
    edge_c: u16,
}

impl MeshDataFace {
    pub fn deconstruct(&self) -> (u16, u16, u16, u16, u16, u16, u16) {
        (
            self.face_id,
            self.vertex_a,
            self.vertex_b,
            self.vertex_c,
            self.edge_a,
            self.edge_b,
            self.edge_c,
        )
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MeshData {
    vertices: Vec<MeshDataVertex>,
    edges: Vec<MeshDataEdge>,
    faces: Vec<MeshDataFace>,
}

impl MeshData {
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

    pub fn get_vertices(&self) -> &Vec<MeshDataVertex> {
        &self.vertices
    }

    pub fn add_vertex(&mut self, x: i16, y: i16, z: i16) {
        self.vertices.push(MeshDataVertex { x, y, z });
    }

    pub fn get_edges(&self) -> &Vec<MeshDataEdge> {
        &self.edges
    }

    pub fn add_edge(&mut self, vertex_a: u16, vertex_b: u16) {
        self.edges.push(MeshDataEdge { vertex_a, vertex_b });
    }

    pub fn get_faces(&self) -> &Vec<MeshDataFace> {
        &self.faces
    }

    pub fn add_face(
        &mut self,
        face_id: u16,
        vertex_a: u16,
        vertex_b: u16,
        vertex_c: u16,
        edge_a: u16,
        edge_b: u16,
        edge_c: u16,
    ) {
        self.faces.push(MeshDataFace {
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
