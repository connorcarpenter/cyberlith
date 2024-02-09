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

// Icon

#[derive(Serialize, Deserialize, Clone)]
pub struct IconFileFrameVertex {
    x: i16,
    y: i16,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct IconFileFrameEdge {
    vertex_a: u16,
    vertex_b: u16,
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

    pub fn add_vertex(&mut self, x: i16, y: i16) {
        self.vertices.push(IconFileFrameVertex { x, y });
    }

    pub fn add_edge(&mut self, start: u16, end: u16) {
        self.edges.push(IconFileFrameEdge {
            vertex_a: start,
            vertex_b: end,
        });
    }

    pub fn add_face(&mut self, face_id: u16, color_id: u8, vertex_a: u16, vertex_b: u16, vertex_c: u16, edge_a: u16, edge_b: u16, edge_c: u16) {
        self.faces.push(IconFileFrameFace {
            face_id,
            color_id,
            vertex_a,
            vertex_b,
            vertex_c,
            edge_a,
            edge_b,
            edge_c,
        });
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct IconFile {
    palette_asset_id: String,
    frames: Vec<IconFileFrame>,
}

impl IconFile {
    pub fn new() -> Self {
        Self {
            palette_asset_id: String::new(),
            frames: Vec::new(),
        }
    }

    pub fn set_palette_asset_id(&mut self, asset_id: &str) {
        self.palette_asset_id = asset_id.to_string();
    }

    pub fn add_frame(&mut self, frame: IconFileFrame) {
        self.frames.push(frame);
    }
}