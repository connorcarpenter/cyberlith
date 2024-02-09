use naia_serde::BitReader;

use asset_io::{AnimAction, IconAction, IconFrameAction, MeshAction, ModelAction, PaletteAction, SceneAction, SerdeQuat, SerdeRotation, SkelAction, SkinAction};
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
pub struct SkelFileVertexParent {
    id: u16,
    rotation: u8,
}

#[derive(Serialize, Deserialize)]
pub struct SkelFileVertex {
    x: i16, y: i16, z: i16, parent: Option<SkelFileVertexParent>, name: Option<String>,
}

impl SkelFileVertex {
    pub fn new(x: i16, y: i16, z: i16, parent_opt: Option<(u16, SerdeRotation)>, name_opt: Option<String>) -> Self {
        let parent = parent_opt
            .map(|(parent_id, rotation)| {
                SkelFileVertexParent {
                    id: parent_id,
                    rotation: rotation.get_inner_value()
                }
            });
        Self {
            x,
            y,
            z,
            parent,
            name: name_opt,
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
    pub vertex_a: u16,
    pub vertex_b: u16,
}

#[derive(Serialize, Deserialize)]
pub struct MeshFileFace {
    pub face_id: u16,
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
                    vertex_a,
                    vertex_b,
                });

            }
            MeshAction::Face(face_id, vertex_a, vertex_b, vertex_c, edge_a, edge_b, edge_c) => {
                file.faces.push(MeshFileFace {
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
    }

    serde_json::to_vec(&file).unwrap()
}

// Animation
#[derive(Serialize, Deserialize)]
pub struct AnimFilePose {
    edge_id: u16,
    quat_x: i8,
    quat_y: i8,
    quat_z: i8,
    quat_w: i8,
}

#[derive(Serialize, Deserialize)]
pub struct AnimFileFrame {
    poses: Vec<AnimFilePose>,
    transition_ms: u16,
}

#[derive(Serialize, Deserialize)]
pub struct AnimFile {
    skeleton_id: String,
    edge_names: Vec<String>,
    frames: Vec<AnimFileFrame>,
}

impl AnimFile {
    pub fn new() -> Self {
        Self {
            skeleton_id: String::new(),
            edge_names: Vec::new(),
            frames: Vec::new(),
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
                file.skeleton_id = format!("{}/{}", path, file_name);
            }
            AnimAction::ShapeIndex(shape_name) => {
                file.edge_names.push(shape_name);
            }
            AnimAction::Frame(poses, transition) => {
                let mut frame = AnimFileFrame {
                    poses: Vec::new(),
                    transition_ms: transition.get_duration_ms(),
                };

                for (shape_index, rotation) in poses {
                    frame.poses.push(AnimFilePose {
                        edge_id: shape_index,
                        quat_x: (rotation.x * SerdeQuat::MAX_SIZE).round() as i8,
                        quat_y: (rotation.y * SerdeQuat::MAX_SIZE).round() as i8,
                        quat_z: (rotation.z * SerdeQuat::MAX_SIZE).round() as i8,
                        quat_w: (rotation.w * SerdeQuat::MAX_SIZE).round() as i8,
                    });
                }

                file.frames.push(frame);
            }
        }
    }

    serde_json::to_vec(&file).unwrap()
}

// Icon
#[derive(Serialize, Deserialize)]
pub struct IconFileFrameVertex {
    pub x: i16,
    pub y: i16,
}


#[derive(Serialize, Deserialize)]
pub struct IconFileFrameEdge {
    pub vertex_a: u16,
    pub vertex_b: u16,
}

#[derive(Serialize, Deserialize)]
pub struct IconFileFrameFace {
    pub face_id: u16,
    pub color_id: u8,
    pub vertex_a: u16,
    pub vertex_b: u16,
    pub vertex_c: u16,
    pub edge_a: u16,
    pub edge_b: u16,
    pub edge_c: u16,
}

#[derive(Serialize, Deserialize)]
pub struct IconFileFrame {
    pub vertices: Vec<IconFileFrameVertex>,
    pub edges: Vec<IconFileFrameEdge>,
    pub faces: Vec<IconFileFrameFace>,
}

impl IconFileFrame {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            edges: Vec::new(),
            faces: Vec::new(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct IconFile {
    palette_id: String,
    frames: Vec<IconFileFrame>,
}

impl IconFile {
    pub fn new() -> Self {
        Self {
            palette_id: String::new(),
            frames: Vec::new(),
        }
    }
}

pub fn icon(in_bytes: &Vec<u8>) -> Vec<u8> {
    let mut bit_reader = BitReader::new(in_bytes);
    let actions = IconAction::read(&mut bit_reader).unwrap();

    let mut file = IconFile::new();

    for action in actions {
        match action {
            IconAction::PaletteFile(path, file_name) => {
                file.palette_id = format!("{}/{}", path, file_name);
            }
            IconAction::Frame(frame_actions) => {
                let mut new_frame = IconFileFrame::new();

                for frame_action in frame_actions {
                    match frame_action {
                        IconFrameAction::Vertex(x, y) => {
                            new_frame.vertices.push(IconFileFrameVertex {
                                x,
                                y,
                            });
                        }
                        IconFrameAction::Edge(start, end) => {
                            new_frame.edges.push(IconFileFrameEdge {
                                vertex_a: start,
                                vertex_b: end,
                            });
                        }
                        IconFrameAction::Face(
                            face_index,
                            palette_color_index,
                            vertex_a_index,
                            vertex_b_index,
                            vertex_c_index,
                            edge_a_index,
                            edge_b_index,
                            edge_c_index
                        ) => {
                            new_frame.faces.push(IconFileFrameFace {
                                face_id: face_index,
                                color_id: palette_color_index,
                                vertex_a: vertex_a_index,
                                vertex_b: vertex_b_index,
                                vertex_c: vertex_c_index,
                                edge_a: edge_a_index,
                                edge_b: edge_b_index,
                                edge_c: edge_c_index,
                            });
                        }
                    }
                }

                file.frames.push(new_frame);
            }
        }
    }

    serde_json::to_vec(&file).unwrap()
}

// Skin
#[derive(Serialize, Deserialize)]
pub struct SkinFileFace {
    face_id: u16,
    color_id: u8,
}

#[derive(Serialize, Deserialize)]
pub struct SkinFile {
    palette_id: String,
    mesh_id: String,
    background_color_id: u8,
    face_colors: Vec<SkinFileFace>,
}

impl SkinFile {
    pub fn new() -> Self {
        Self {
            palette_id: String::new(),
            mesh_id: String::new(),
            background_color_id: 0,
            face_colors: Vec::new(),
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
                file.palette_id = format!("{}/{}", path, file_name);
            }
            SkinAction::MeshFile(path, file_name) => {
                file.mesh_id = format!("{}/{}", path, file_name);
            }
            SkinAction::BackgroundColor(palette_color_id) => {
                file.background_color_id = palette_color_id;
            }
            SkinAction::SkinColor(face_id, color_id) => {
                file.face_colors.push(SkinFileFace {
                    face_id,
                    color_id,
                });
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