use cfg_if::cfg_if;

use serde::{Deserialize, Serialize};
use crate::AssetId;

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
pub struct PaletteFile {
    colors: Vec<PaletteFileColor>,
}

impl PaletteFile {

    pub const CURRENT_SCHEMA_VERSION: u32 = 0;

    pub fn new() -> Self {
        Self {
            colors: Vec::new(),
        }
    }

    pub fn dependencies(&self) -> Vec<AssetId> {
        Vec::new()
    }

    pub fn get_colors(&self) -> &Vec<PaletteFileColor> {
        &self.colors
    }

    pub fn add_color(&mut self, r: u8, g: u8, b: u8) {
        self.colors.push(PaletteFileColor {
            r,
            g,
            b,
        });
    }

    pub fn insert_color(&mut self, index: usize, r: u8, g: u8, b: u8) {
        if index >= self.colors.len() {
            self.colors.resize(index + 1, PaletteFileColor { r: 0, g: 0, b: 0 });
        }
        self.colors[index] = PaletteFileColor { r, g, b };
    }
}