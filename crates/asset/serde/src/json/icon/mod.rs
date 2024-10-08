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

// Icon

#[derive(Serialize, Deserialize, Clone)]
pub struct IconFileFrameVertex {
    x: i16,
    y: i16,
}

impl IconFileFrameVertex {
    pub fn new(x: i16, y: i16) -> Self {
        Self { x, y }
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
        Self { vertex_a, vertex_b }
    }

    pub fn vertex_a(&self) -> u16 {
        self.vertex_a
    }

    pub fn vertex_b(&self) -> u16 {
        self.vertex_b
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub struct IconFileFrameFace {
    face_id: u16,
    color_id: u8,
    vertex_a: u16,
    vertex_b: u16,
    vertex_c: u16,
}

// impl Serialize for IconFileFrameFace {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
//         // 3 is the number of fields in the struct.
//         let mut state = serializer.serialize_struct("IconFileFrameFace", 5)?;
//         state.serialize_field("face_id", &self.face_id)?;
//         state.serialize_field("color_id", &self.color_id)?;
//         state.serialize_field("vertex_a", &self.vertex_a)?;
//         state.serialize_field("vertex_b", &self.vertex_b)?;
//         state.serialize_field("vertex_c", &self.vertex_c)?;
//         state.end()
//     }
// }

impl IconFileFrameFace {
    pub fn new(face_id: u16, color_id: u8, vertex_a: u16, vertex_b: u16, vertex_c: u16) -> Self {
        Self {
            face_id,
            color_id,
            vertex_a,
            vertex_b,
            vertex_c,
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
        self.edges.push(IconFileFrameEdge::new(start, end));
    }

    pub fn get_faces(&self) -> &Vec<IconFileFrameFace> {
        &self.faces
    }

    pub fn add_face(
        &mut self,
        face_id: u16,
        color_id: u8,
        vertex_a: u16,
        vertex_b: u16,
        vertex_c: u16,
    ) {
        self.faces.push(IconFileFrameFace::new(
            face_id, color_id, vertex_a, vertex_b, vertex_c,
        ));
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct IconJson {
    palette_asset_id: String,
    frames: Vec<IconFileFrame>,
}

impl IconJson {
    pub const CURRENT_SCHEMA_VERSION: u32 = 0;

    pub fn new() -> Self {
        Self {
            palette_asset_id: String::new(),
            frames: Vec::new(),
        }
    }

    pub fn dependencies(&self) -> Vec<AssetId> {
        let mut output = Vec::new();

        let Some(asset_id) = self.get_palette_asset_id() else {
            panic!("icon missing palette");
        };
        output.push(asset_id);

        output
    }

    pub fn get_palette_asset_id(&self) -> Option<AssetId> {
        AssetId::from_str(&self.palette_asset_id).ok()
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
