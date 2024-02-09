use cfg_if::cfg_if;

use serde::{Deserialize, Serialize};

cfg_if! {
    if #[cfg(feature = "read")] {
        mod read;
        pub use read::*;
    } else {}
}

cfg_if! {
    if #[cfg(feature = "write")] {
        mod write;
        pub use write::*;
    } else {}
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
    pub fn set_palette_asset_id(&mut self, asset_id: &str) {
        self.palette_asset_id = asset_id.to_string();
    }

    pub fn set_mesh_asset_id(&mut self, asset_id: &str) {
        self.mesh_asset_id = asset_id.to_string();
    }

    pub fn set_background_color_id(&mut self, background_color_id: u8) {
        self.background_color_id = background_color_id;
    }

    pub fn add_face_color(&mut self, face_id: u16, color_id: u8) {
        self.face_colors.push(SkinFileFace {
            face_id,
            color_id,
        });
    }
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