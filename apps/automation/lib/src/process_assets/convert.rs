use std::collections::HashMap;

use asset_io::{AnimAction, FileTransformEntityType, IconAction, IconFrameAction, MeshAction, ModelAction, PaletteAction, SceneAction, SerdeQuat, SerdeRotation, SkelAction, SkinAction};
use serde::{Deserialize, Serialize};

use crate::process_assets::json::ProcessData;

#[derive(Serialize, Deserialize, Clone)]
pub struct Asset {
    pub meta: AssetMeta,
    pub data: AssetData,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AssetMeta {
    pub asset_id: String,
    pub schema_version: u32,
}

// Container
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum AssetData {
    Palette(PaletteFile),
    Skeleton(SkelFile),
    Mesh(MeshFile),
    Animation(AnimFile),
    Icon(IconFile),
    Skin(SkinFile),
    Scene(SceneFile),
    Model(ModelFile),
}

impl AssetData {
    pub(crate) fn convert_to_asset_ids(&mut self, asset_map: &HashMap<String, ProcessData>) {
        match self {
            Self::Animation(inner) => {
                inner.skeleton_asset_id = asset_map.get(&inner.skeleton_asset_id).unwrap().asset_id.as_string();
            }
            Self::Icon(inner) => {
                inner.palette_asset_id = asset_map.get(&inner.palette_asset_id).unwrap().asset_id.as_string();
            }
            Self::Palette(_) => {
                // Do nothing
            }
            Self::Skeleton(_) => {
                // Do nothing
            }
            Self::Mesh(_) => {
                // Do nothing
            }
            Self::Skin(inner) => {
                inner.palette_asset_id = asset_map.get(&inner.palette_asset_id).unwrap().asset_id.as_string();
                inner.mesh_asset_id = asset_map.get(&inner.mesh_asset_id).unwrap().asset_id.as_string();
            }
            Self::Scene(inner) => {
                for component in &mut inner.components {
                    component.asset_id = asset_map.get(&component.asset_id).unwrap().asset_id.as_string();
                }
            }
            Self::Model(inner) => {
                inner.skeleton_id = asset_map.get(&inner.skeleton_id).unwrap().asset_id.as_string();
                for component in &mut inner.components {
                    component.asset_id = asset_map.get(&component.asset_id).unwrap().asset_id.as_string();
                }
            }
        }
    }
}

// Palette
#[derive(Serialize, Deserialize, Clone)]
pub struct PaletteFileColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Serialize, Deserialize, Clone)]
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

pub fn palette(in_bytes: &Vec<u8>) -> AssetData {
    let actions = PaletteAction::read(in_bytes).unwrap();

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

    AssetData::Palette(file)
}

// Skel
#[derive(Serialize, Deserialize, Clone)]
pub struct SkelFileVertexParent {
    id: u16,
    rotation: u8,
}

#[derive(Serialize, Deserialize, Clone)]
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

#[derive(Serialize, Deserialize, Clone)]
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

pub fn skel(in_bytes: &Vec<u8>) -> AssetData {
    let actions = SkelAction::read(in_bytes).unwrap();

    let mut file = SkelFile::new();

    for action in actions {
        match action {
            SkelAction::Vertex(x, y, z, parent_id_opt, vertex_name_opt) => {
                file.vertices.push(SkelFileVertex::new(x, y, z, parent_id_opt, vertex_name_opt));
            }
        }
    }

    AssetData::Skeleton(file)
}

// Mesh

#[derive(Serialize, Deserialize, Clone)]
pub struct MeshFileVertex {
    pub x: i16,
    pub y: i16,
    pub z: i16,
}


#[derive(Serialize, Deserialize, Clone)]
pub struct MeshFileEdge {
    pub vertex_a: u16,
    pub vertex_b: u16,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MeshFileFace {
    pub face_id: u16,
    pub vertex_a: u16,
    pub vertex_b: u16,
    pub vertex_c: u16,
    pub edge_a: u16,
    pub edge_b: u16,
    pub edge_c: u16,
}

#[derive(Serialize, Deserialize, Clone)]
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

pub fn mesh(in_bytes: &Vec<u8>) -> AssetData {
    let actions = MeshAction::read(in_bytes).unwrap();

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

    AssetData::Mesh(file)
}

// Animation
#[derive(Serialize, Deserialize, Clone)]
pub struct AnimFileQuat {
    x: i8,
    y: i8,
    z: i8,
    w: i8,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AnimFilePose {
    edge_id: u16,
    rotation: AnimFileQuat,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AnimFileFrame {
    poses: Vec<AnimFilePose>,
    transition_ms: u16,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AnimFile {
    skeleton_asset_id: String,
    edge_names: Vec<String>,
    frames: Vec<AnimFileFrame>,
}

impl AnimFile {
    pub fn new() -> Self {
        Self {
            skeleton_asset_id: String::new(),
            edge_names: Vec::new(),
            frames: Vec::new(),
        }
    }
}

pub fn anim(in_bytes: &Vec<u8>) -> AssetData {
    let actions = AnimAction::read(in_bytes).unwrap();

    let mut file = AnimFile::new();

    for action in actions {
        match action {
            AnimAction::SkelFile(path, file_name) => {
                file.skeleton_asset_id = format!("{}/{}", path, file_name);
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
                        rotation: AnimFileQuat {
                            x: (rotation.x * SerdeQuat::MAX_SIZE).round() as i8,
                            y: (rotation.y * SerdeQuat::MAX_SIZE).round() as i8,
                            z: (rotation.z * SerdeQuat::MAX_SIZE).round() as i8,
                            w: (rotation.w * SerdeQuat::MAX_SIZE).round() as i8,
                        },
                    });
                }

                file.frames.push(frame);
            }
        }
    }

    AssetData::Animation(file)
}

// Icon
#[derive(Serialize, Deserialize, Clone)]
pub struct IconFileFrameVertex {
    pub x: i16,
    pub y: i16,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct IconFileFrameEdge {
    pub vertex_a: u16,
    pub vertex_b: u16,
}

#[derive(Serialize, Deserialize, Clone)]
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

#[derive(Serialize, Deserialize, Clone)]
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
}

pub fn icon(in_bytes: &Vec<u8>) -> AssetData {
    let actions = IconAction::read(in_bytes).unwrap();

    let mut file = IconFile::new();

    for action in actions {
        match action {
            IconAction::PaletteFile(path, file_name) => {
                file.palette_asset_id = format!("{}/{}", path, file_name);
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

    AssetData::Icon(file)
}

// Skin
#[derive(Serialize, Deserialize, Clone)]
pub struct SkinFileFace {
    face_id: u16,
    color_id: u8,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SkinFile {
    palette_asset_id: String,
    mesh_asset_id: String,
    background_color_id: u8,
    face_colors: Vec<SkinFileFace>,
}

impl SkinFile {
    pub fn new() -> Self {
        Self {
            palette_asset_id: String::new(),
            mesh_asset_id: String::new(),
            background_color_id: 0,
            face_colors: Vec::new(),
        }
    }
}

pub fn skin(in_bytes: &Vec<u8>) -> AssetData {
    let actions = SkinAction::read(in_bytes).unwrap();

    let mut file = SkinFile::new();

    for action in actions {
        match action {
            SkinAction::PaletteFile(path, file_name) => {
                file.palette_asset_id = format!("{}/{}", path, file_name);
            }
            SkinAction::MeshFile(path, file_name) => {
                file.mesh_asset_id = format!("{}/{}", path, file_name);
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

    AssetData::Skin(file)
}

// Scene
#[derive(Serialize, Deserialize, Clone)]
pub struct SceneFileComponent {
    asset_id: String,
    kind: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SceneFileTransform {
    component_id: u16,
    position: SceneFileTransformPosition,
    rotation: SceneFileTransformRotation,
    scale: SceneFileTransformScale,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SceneFileTransformPosition {
    x: i16, y: i16, z: i16,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SceneFileTransformRotation {
    x: i8, y: i8, z: i8, w: i8,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SceneFileTransformScale {
    x: u32, y: u32, z: u32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SceneFile {
    components: Vec<SceneFileComponent>,
    transforms: Vec<SceneFileTransform>,
}

impl SceneFile {
    pub fn new() -> Self {
        Self {
            components: Vec::new(),
            transforms: Vec::new(),
        }
    }
}

pub fn scene(in_bytes: &Vec<u8>) -> AssetData {
    let actions = SceneAction::read(in_bytes).unwrap();

    let mut file = SceneFile::new();

    for action in actions {
        match action {
            SceneAction::SkinOrSceneFile(path, file_name, file_type) => {
                file.components.push(SceneFileComponent {
                    asset_id: format!("{}/{}", path, file_name),
                    kind: match file_type {
                        FileTransformEntityType::Skin => "skin".to_string(),
                        FileTransformEntityType::Scene => "scene".to_string(),
                    },
                });
            }
            SceneAction::NetTransform(
                file_id,
                x,
                y,
                z,
                scale_x,
                scale_y,
                scale_z,
                rotation
            ) => {
                let transform = SceneFileTransform {
                    component_id: file_id,
                    position: SceneFileTransformPosition {
                        x,
                        y,
                        z,
                    },
                    rotation: SceneFileTransformRotation {
                        x: (rotation.x * SerdeQuat::MAX_SIZE).round() as i8,
                        y: (rotation.y * SerdeQuat::MAX_SIZE).round() as i8,
                        z: (rotation.z * SerdeQuat::MAX_SIZE).round() as i8,
                        w: (rotation.w * SerdeQuat::MAX_SIZE).round() as i8,
                    },
                    scale: SceneFileTransformScale {
                        x: (scale_x * 100.0) as u32,
                        y: (scale_y * 100.0) as u32,
                        z: (scale_z * 100.0) as u32,
                    },
                };
                file.transforms.push(transform);
            }
        }
    }

    AssetData::Scene(file)
}

// Model
#[derive(Serialize, Deserialize, Clone)]
pub struct ModelFileComponent {
    asset_id: String,
    kind: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ModelFileTransform {
    component_id: u16,
    name: String,
    position: ModelFileTransformPosition,
    rotation: ModelFileTransformRotation,
    scale: ModelFileTransformScale,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ModelFileTransformPosition {
    x: i16, y: i16, z: i16,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ModelFileTransformRotation {
    x: i8, y: i8, z: i8, w: i8,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ModelFileTransformScale {
    x: u32, y: u32, z: u32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ModelFile {
    skeleton_id: String,
    components: Vec<ModelFileComponent>,
    transforms: Vec<ModelFileTransform>,
}

impl ModelFile {
    pub fn new() -> Self {
        Self {
            skeleton_id: String::new(),
            components: Vec::new(),
            transforms: Vec::new(),
        }
    }
}

pub fn model(in_bytes: &Vec<u8>) -> AssetData {
    let actions = ModelAction::read(in_bytes).unwrap();

    let mut file = ModelFile::new();

    for action in actions {
        match action {
            ModelAction::SkelFile(path, file_name) => {
                file.skeleton_id = format!("{}/{}", path, file_name);
            }
            ModelAction::SkinOrSceneFile(path, file_name, file_type) => {
                file.components.push(ModelFileComponent {
                    asset_id: format!("{}/{}", path, file_name),
                    kind: match file_type {
                        FileTransformEntityType::Skin => "skin".to_string(),
                        FileTransformEntityType::Scene => "scene".to_string(),
                    },
                });
            }
            ModelAction::NetTransform(
                skin_index,
                vertex_name,
                translation_x,
                translation_y,
                translation_z,
                scale_x,
                scale_y,
                scale_z,
                rotation
            ) => {
                let transform = ModelFileTransform {
                    component_id: skin_index,
                    name: vertex_name,
                    position: ModelFileTransformPosition {
                        x: translation_x,
                        y: translation_y,
                        z: translation_z,
                    },
                    rotation: ModelFileTransformRotation {
                        x: (rotation.x * SerdeQuat::MAX_SIZE).round() as i8,
                        y: (rotation.y * SerdeQuat::MAX_SIZE).round() as i8,
                        z: (rotation.z * SerdeQuat::MAX_SIZE).round() as i8,
                        w: (rotation.w * SerdeQuat::MAX_SIZE).round() as i8,
                    },
                    scale: ModelFileTransformScale {
                        x: (scale_x * 100.0) as u32,
                        y: (scale_y * 100.0) as u32,
                        z: (scale_z * 100.0) as u32,
                    },
                };
                file.transforms.push(transform);
            }
        }
    }

    AssetData::Model(file)
}