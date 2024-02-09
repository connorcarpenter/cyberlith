use naia_serde::BitReader;

use asset_io::{AnimAction, IconAction, MeshAction, ModelAction, PaletteAction, SceneAction, SerdeRotation, SkelAction, SkinAction};
use serde::{Deserialize, Serialize};

// Palette
#[derive(Serialize, Deserialize)]
pub struct PaletteFileColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Serialize, Deserialize)]
pub struct PaletteFile {
    pub colors: Vec<PaletteFileColor>,
}

impl PaletteFile {
    pub fn new() -> Self {
        Self {
            colors: Vec::new(),
        }
    }
}

pub fn palette(in_bytes: &Vec<u8>) -> Vec<u8> {
    let mut bit_reader = BitReader::new(in_bytes);
    let actions = PaletteAction::read(&mut bit_reader).unwrap();

    let mut file = PaletteFile::new();

    for action in actions {
        match action {
            PaletteAction::Color(r, g, b) => {
                file.colors.push(PaletteFileColor {
                    r,
                    g,
                    b,
                });
            }
        }
    }

    serde_json::to_vec(&file).unwrap()
}

// Skel
#[derive(Serialize, Deserialize)]
pub struct SkelFileVertex {
    x: i16, y: i16, z: i16, parent_opt: Option<(u16, u8)>, name_opt: Option<String>,
}

impl SkelFileVertex {
    pub fn new(x: i16, y: i16, z: i16, parent_opt: Option<(u16, SerdeRotation)>, name_opt: Option<String>) -> Self {
        let parent_opt = parent_opt.map(|(parent_id, rotation)| (parent_id, rotation.get_inner_value()));
        Self {
            x,
            y,
            z,
            parent_opt,
            name_opt,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct SkelFile {
    pub vertices: Vec<SkelFileVertex>,
}

impl SkelFile {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
        }
    }
}

pub fn skel(in_bytes: &Vec<u8>) -> Vec<u8> {
    let mut bit_reader = BitReader::new(in_bytes);
    let actions = SkelAction::read(&mut bit_reader).unwrap();

    let mut file = SkelFile::new();

    for action in actions {
        match action {
            SkelAction::Vertex(x, y, z, parent_id_opt, vertex_name_opt) => {
                file.vertices.push(SkelFileVertex::new(x, y, z, parent_id_opt, vertex_name_opt));
            }
        }
    }

    serde_json::to_vec(&file).unwrap()
}

// Mesh

#[derive(Serialize, Deserialize)]
pub struct MeshFileVertex {
    pub x: i16,
    pub y: i16,
    pub z: i16,
}


#[derive(Serialize, Deserialize)]
pub struct MeshFileEdge {
    pub start: u16,
    pub end: u16,
}


#[derive(Serialize, Deserialize)]
pub struct MeshFileFace {
    pub face_index: u16,
    pub vertex_a: u16,
    pub vertex_b: u16,
    pub vertex_c: u16,
    pub edge_a: u16,
    pub edge_b: u16,
    pub edge_c: u16,
}

#[derive(Serialize, Deserialize)]
pub struct MeshFile {
    pub vertices: Vec<MeshFileVertex>,
    pub edges: Vec<MeshFileEdge>,
    pub faces: Vec<MeshFileFace>,
}

impl MeshFile {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            edges: Vec::new(),
            faces: Vec::new(),
        }
    }
}

pub fn mesh(in_bytes: &Vec<u8>) -> Vec<u8> {
    let mut bit_reader = BitReader::new(in_bytes);
    let actions = MeshAction::read(&mut bit_reader).unwrap();

    let mut file = MeshFile::new();

    for action in actions {
        match action {
            MeshAction::Vertex(x, y, z) => {
                file.vertices.push(MeshFileVertex {
                    x,
                    y,
                    z,
                });
            }
            MeshAction::Edge(vertex_a, vertex_b) => {
                file.edges.push(MeshFileEdge {
                    start: vertex_a,
                    end: vertex_b,
                });

            }
            MeshAction::Face(face_index, vertex_a, vertex_b, vertex_c, edge_a, edge_b, edge_c) => {
                file.faces.push(MeshFileFace {
                    face_index,
                    vertex_a,
                    vertex_b,
                    vertex_c,
                    edge_a,
                    edge_b,
                    edge_c,
                });
            }
        }
    }

    serde_json::to_vec(&file).unwrap()
}

// Animation
#[derive(Serialize, Deserialize)]
pub struct AnimFile {

}

impl AnimFile {
    pub fn new() -> Self {
        Self {

        }
    }
}

pub fn anim(in_bytes: &Vec<u8>) -> Vec<u8> {
    let mut bit_reader = BitReader::new(in_bytes);
    let actions = AnimAction::read(&mut bit_reader).unwrap();

    let mut file = AnimFile::new();

    for action in actions {
        match action {
            AnimAction::SkelFile(path, file_name) => {
                todo!();
            }
            AnimAction::ShapeIndex(shape_name) => {
                todo!();
            }
            AnimAction::Frame(poses, transition) => {
                todo!();
            }
        }
    }

    serde_json::to_vec(&file).unwrap()
}

// Icon
#[derive(Serialize, Deserialize)]
pub struct IconFile {

}

impl IconFile {
    pub fn new() -> Self {
        Self {

        }
    }
}

pub fn icon(in_bytes: &Vec<u8>) -> Vec<u8> {
    let mut bit_reader = BitReader::new(in_bytes);
    let actions = IconAction::read(&mut bit_reader).unwrap();

    let mut file = IconFile::new();

    for action in actions {
        match action {
            IconAction::PaletteFile(palette_path, palette_file_name) => {
                todo!();
            }
            IconAction::Frame(frame_actions) => {
                todo!();
            }
        }
    }

    serde_json::to_vec(&file).unwrap()
}

// Skin
#[derive(Serialize, Deserialize)]
pub struct SkinFile {

}

impl SkinFile {
    pub fn new() -> Self {
        Self {

        }
    }
}

pub fn skin(in_bytes: &Vec<u8>) -> Vec<u8> {
    let mut bit_reader = BitReader::new(in_bytes);
    let actions = SkinAction::read(&mut bit_reader).unwrap();

    let mut file = SkinFile::new();

    for action in actions {
        match action {
            SkinAction::PaletteFile(path, file_name) => {
                todo!();
            }
            SkinAction::MeshFile(path, file_name) => {
                todo!();
            }
            SkinAction::BackgroundColor(palette_color_index) => {
                todo!();
            }
            SkinAction::SkinColor(face_index, palette_color_index) => {
                todo!();
            }
        }
    }

    serde_json::to_vec(&file).unwrap()
}

// Scene
#[derive(Serialize, Deserialize)]
pub struct SceneFile {

}

impl SceneFile {
    pub fn new() -> Self {
        Self {

        }
    }
}

pub fn scene(in_bytes: &Vec<u8>) -> Vec<u8> {
    let mut bit_reader = BitReader::new(in_bytes);
    let actions = SceneAction::read(&mut bit_reader).unwrap();

    let mut file = SceneFile::new();

    for action in actions {
        match action {
            SceneAction::SkinOrSceneFile(path, file_name, file_type) => {
                todo!()
            }
            SceneAction::NetTransform(file_index,
                                      x,
                                      y,
                                      z,
                                      scale_x,
                                      scale_y,
                                      scale_z,
                                      rotation) => {
                todo!()
            }
        }
    }

    serde_json::to_vec(&file).unwrap()
}

// Model
#[derive(Serialize, Deserialize)]
pub struct ModelFile {

}

impl ModelFile {
    pub fn new() -> Self {
        Self {

        }
    }
}

pub fn model(in_bytes: &Vec<u8>) -> Vec<u8> {
    let mut bit_reader = BitReader::new(in_bytes);
    let actions = ModelAction::read(&mut bit_reader).unwrap();

    let mut file = ModelFile::new();

    for action in actions {
        match action {
            ModelAction::SkelFile(path, file_name) => {
                todo!()
            }
            ModelAction::SkinOrSceneFile(path, file_name, file_type) => {
                todo!()
            }
            ModelAction::NetTransform(skin_index,
                                      vertex_name,
                                      translation_x,
                                      translation_y,
                                      translation_z,
                                      scale_x,
                                      scale_y,
                                      scale_z,
                                      rotation) => {
                todo!()
            }
        }
    }

    serde_json::to_vec(&file).unwrap()
}