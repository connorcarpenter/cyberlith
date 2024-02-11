use cfg_if::cfg_if;

use serde::{Deserialize, Serialize};

use crate::json::AssetId;

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

// Icon

#[derive(Serialize, Deserialize, Clone)]
pub struct IconFileFrameVertex {
    x: i16,
    y: i16,
}

impl IconFileFrameVertex {
    pub fn new(x: i16, y: i16) -> Self {
        Self {
            x,
            y,
        }
    }

    pub fn x(&self) -> i16 {
        self.x
    }

    pub fn y(&self) -> i16 {
        self.y
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct IconFileFrameEdge {
    vertex_a: u16,
    vertex_b: u16,
}

impl IconFileFrameEdge {
    pub fn new(vertex_a: u16, vertex_b: u16) -> Self {
        Self {
            vertex_a,
            vertex_b,
        }
    }

    pub fn vertex_a(&self) -> u16 {
        self.vertex_a
    }

    pub fn vertex_b(&self) -> u16 {
        self.vertex_b
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct IconFileFrameFace {
    face_id: u16,
    color_id: u8,
    vertex_a: u16,
    vertex_b: u16,
    vertex_c: u16,
    edge_a: u16,
    edge_b: u16,
    edge_c: u16,
}

impl IconFileFrameFace {
    pub fn new(face_id: u16, color_id: u8, vertex_a: u16, vertex_b: u16, vertex_c: u16, edge_a: u16, edge_b: u16, edge_c: u16) -> Self {
        Self {
            face_id,
            color_id,
            vertex_a,
            vertex_b,
            vertex_c,
            edge_a,
            edge_b,
            edge_c,
        }
    }

    pub fn face_id(&self) -> u16 {
        self.face_id
    }

    pub fn color_id(&self) -> u8 {
        self.color_id
    }

    pub fn vertex_a(&self) -> u16 {
        self.vertex_a
    }

    pub fn vertex_b(&self) -> u16 {
        self.vertex_b
    }

    pub fn vertex_c(&self) -> u16 {
        self.vertex_c
    }

    pub fn edge_a(&self) -> u16 {
        self.edge_a
    }

    pub fn edge_b(&self) -> u16 {
        self.edge_b
    }

    pub fn edge_c(&self) -> u16 {
        self.edge_c
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct IconFileFrame {
    vertices: Vec<IconFileFrameVertex>,
    edges: Vec<IconFileFrameEdge>,
    faces: Vec<IconFileFrameFace>,
}

impl IconFileFrame {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            edges: Vec::new(),
            faces: Vec::new(),
        }
    }

    pub fn get_vertices(&self) -> &Vec<IconFileFrameVertex> {
        &self.vertices
    }

    pub fn add_vertex(&mut self, x: i16, y: i16) {
        self.vertices.push(IconFileFrameVertex::new(x, y));
    }

    pub fn get_edges(&self) -> &Vec<IconFileFrameEdge> {
        &self.edges
    }

    pub fn add_edge(&mut self, start: u16, end: u16) {
        self.edges.push(IconFileFrameEdge::new(
            start,
            end,
        ));
    }

    pub fn get_faces(&self) -> &Vec<IconFileFrameFace> {
        &self.faces
    }

    pub fn add_face(&mut self, face_id: u16, color_id: u8, vertex_a: u16, vertex_b: u16, vertex_c: u16, edge_a: u16, edge_b: u16, edge_c: u16) {
        self.faces.push(IconFileFrameFace::new(
            face_id,
            color_id,
            vertex_a,
            vertex_b,
            vertex_c,
            edge_a,
            edge_b,
            edge_c,
        ));
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct IconFile {
    palette_asset_id: String,
    frames: Vec<IconFileFrame>,
}

impl IconFile {

    pub const CURRENT_SCHEMA_VERSION: u32 = 0;

    pub fn new() -> Self {
        Self {
            palette_asset_id: String::new(),
            frames: Vec::new(),
        }
    }

    pub fn get_palette_asset_id(&self) -> AssetId {
        AssetId::from_str(&self.palette_asset_id).unwrap()
    }

    pub fn set_palette_asset_id(&mut self, asset_id: &AssetId) {
        self.palette_asset_id = asset_id.as_string();
    }

    pub fn get_frames(&self) -> &Vec<IconFileFrame> {
        &self.frames
    }

    pub fn add_frame(&mut self, frame: IconFileFrame) {
        self.frames.push(frame);
    }
}