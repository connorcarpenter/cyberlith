use cfg_if::cfg_if;

use serde::{Deserialize, Serialize};
use asset_id::AssetId;

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

// Skin

#[derive(Serialize, Deserialize, Clone)]
pub struct SkinFileFace {
    face_id: u16,
    color_id: u8,
}

impl SkinFileFace {
    pub fn new(face_id: u16, color_id: u8) -> Self {
        Self {
            face_id,
            color_id,
        }
    }

    pub fn face_id(&self) -> u16 {
        self.face_id
    }

    pub fn color_id(&self) -> u8 {
        self.color_id
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SkinFile {
    palette_asset_id: String,
    mesh_asset_id: String,
    background_color_id: u8,
    face_colors: Vec<SkinFileFace>,
}

impl SkinFile {

    pub const CURRENT_SCHEMA_VERSION: u32 = 0;

    pub fn new() -> Self {
        Self {
            palette_asset_id: String::new(),
            mesh_asset_id: String::new(),
            background_color_id: 0,
            face_colors: Vec::new(),
        }
    }

    pub fn dependencies(&self) -> Vec<AssetId> {
        let mut output = Vec::new();

        output.push(self.get_palette_asset_id());
        output.push(self.get_mesh_asset_id());

        output
    }

    pub fn get_palette_asset_id(&self) -> AssetId {
        AssetId::from_str(self.palette_asset_id.as_str()).unwrap()
    }

    pub fn set_palette_asset_id(&mut self, asset_id: &AssetId) {
        self.palette_asset_id = asset_id.as_string();
    }

    pub fn get_mesh_asset_id(&self) -> AssetId {
        AssetId::from_str(self.mesh_asset_id.as_str()).unwrap()
    }

    pub fn set_mesh_asset_id(&mut self, asset_id: &AssetId) {
        self.mesh_asset_id = asset_id.as_string();
    }

    pub fn get_background_color_id(&self) -> u8 {
        self.background_color_id
    }

    pub fn set_background_color_id(&mut self, background_color_id: u8) {
        self.background_color_id = background_color_id;
    }

    pub fn get_face_colors(&self) -> &Vec<SkinFileFace> {
        &self.face_colors
    }

    pub fn add_face_color(&mut self, face_id: u16, color_id: u8) {
        self.face_colors.push(SkinFileFace {
            face_id,
            color_id,
        });
    }
}