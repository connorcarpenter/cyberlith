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

// Palette

#[derive(Serialize, Deserialize, Clone)]
pub struct PaletteFileColor {
    r: u8,
    g: u8,
    b: u8,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PaletteFile {
    colors: Vec<PaletteFileColor>,
}

impl PaletteFile {
    pub fn new() -> Self {
        Self {
            colors: Vec::new(),
        }
    }

    pub fn add_color(&mut self, r: u8, g: u8, b: u8) {
        self.colors.push(PaletteFileColor {
            r,
            g,
            b,
        });
    }
}