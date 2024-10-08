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

// Palette

#[derive(Serialize, Deserialize, Clone)]
pub struct PaletteFileColor {
    r: u8,
    g: u8,
    b: u8,
}

impl PaletteFileColor {
    pub fn deconstruct(&self) -> (u8, u8, u8) {
        (self.r, self.g, self.b)
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PaletteJson {
    colors: Vec<PaletteFileColor>,
}

impl PaletteJson {
    pub const CURRENT_SCHEMA_VERSION: u32 = 0;

    pub fn new() -> Self {
        Self { colors: Vec::new() }
    }

    pub fn dependencies(&self) -> Vec<AssetId> {
        Vec::new()
    }

    pub fn get_colors(&self) -> &Vec<PaletteFileColor> {
        &self.colors
    }

    pub fn add_color(&mut self, r: u8, g: u8, b: u8) {
        self.colors.push(PaletteFileColor { r, g, b });
    }

    pub fn insert_color(&mut self, index: usize, r: u8, g: u8, b: u8) {
        if index >= self.colors.len() {
            self.colors
                .resize(index + 1, PaletteFileColor { r: 0, g: 0, b: 0 });
        }
        self.colors[index] = PaletteFileColor { r, g, b };
    }
}
