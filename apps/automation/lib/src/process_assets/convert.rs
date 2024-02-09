use naia_serde::BitReader;

use asset_io::{AnimAction, IconAction, MeshAction, ModelAction, PaletteAction, SceneAction, SkelAction, SkinAction};
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
pub struct SkelFile {

}

impl SkelFile {
    pub fn new() -> Self {
        Self {

        }
    }
}

pub fn skel(in_bytes: &Vec<u8>) -> Vec<u8> {
    let mut bit_reader = BitReader::new(in_bytes);
    let actions = SkelAction::read(&mut bit_reader).unwrap();

    let mut file = SkelFile::new();

    for action in actions {
        match action {

        }
    }

    serde_json::to_vec(&file).unwrap()
}

// Mesh
#[derive(Serialize, Deserialize)]
pub struct MeshFile {

}

impl MeshFile {
    pub fn new() -> Self {
        Self {

        }
    }
}

pub fn mesh(in_bytes: &Vec<u8>) -> Vec<u8> {
    let mut bit_reader = BitReader::new(in_bytes);
    let actions = MeshAction::read(&mut bit_reader).unwrap();

    let mut file = MeshFile::new();

    for action in actions {
        match action {

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

        }
    }

    serde_json::to_vec(&file).unwrap()
}