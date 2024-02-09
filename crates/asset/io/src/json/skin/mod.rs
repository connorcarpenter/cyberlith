use cfg_if::cfg_if;

use serde::{Deserialize, Serialize};
use crypto::U32Token;

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

    pub const CURRENT_SCHEMA_VERSION: u32 = 1;

    pub fn new() -> Self {
        Self {
            palette_asset_id: String::new(),
            mesh_asset_id: String::new(),
            background_color_id: 0,
            face_colors: Vec::new(),
        }
    }

    pub fn set_palette_asset_id(&mut self, asset_id: &U32Token) {
        self.palette_asset_id = asset_id.as_string();
    }

    pub fn set_mesh_asset_id(&mut self, asset_id: &U32Token) {
        self.mesh_asset_id = asset_id.as_string();
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