use math::*;

use crate::{assets::AssetHash, base::CpuMaterial};

/// Represents a color composed of a red, green and blue component.
/// In addition, the alpha value determines the how transparent the color is (0 is fully transparent and 255 is fully opaque).
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Color {
    /// Red component
    pub r: u8,
    /// Green component
    pub g: u8,
    /// Blue component
    pub b: u8,
}

impl AssetHash<CpuMaterial> for Color {}

impl Color {
    /// Opaque red
    pub const RED: Color = Color::new(255, 0, 0);
    /// Opaque green
    pub const GREEN: Color = Color::new(0, 255, 0);
    /// Opaque light green
    pub const LIGHT_GREEN: Color = Color::new(16, 255, 16);
    /// Opaque dark green
    pub const DARK_GREEN: Color = Color::new(0, 16, 0);
    /// Opaque blue
    pub const BLUE: Color = Color::new(0, 64, 255);
    /// Opaque light blue
    pub const LIGHT_BLUE: Color = Color::new(0, 32, 255);
    /// Opaque dark blue
    pub const DARK_BLUE: Color = Color::new(0, 48, 192);
    /// Opaque aqua
    pub const AQUA: Color = Color::new(0, 255, 255);
    /// Opaque white
    pub const WHITE: Color = Color::new(255, 255, 255);
    /// Opaque black
    pub const BLACK: Color = Color::new(0, 0, 0);
    /// Opaque dark gray
    pub const LIGHT_GRAY: Color = Color::new(64, 64, 64);
    /// Opaque gray
    pub const GRAY: Color = Color::new(40, 40, 40);
    /// Opaque light gray
    pub const DARK_GRAY: Color = Color::new(16, 16, 16);

    ///
    /// Creates a new color with the given red, green and blue values
    ///
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    ///
    /// Creates a new color from three float elements where each element are in the range `0.0..=1.0`.
    ///
    pub fn from_rgb_slice(rgb: &[f32; 3]) -> Self {
        Self {
            r: (rgb[0] * 255.0) as u8,
            g: (rgb[1] * 255.0) as u8,
            b: (rgb[2] * 255.0) as u8,
            ..Default::default()
        }
    }

    /// Convert to [`Vec3`] by mapping the red, green and blue component to the range `0.0..=1.0`.
    pub fn to_vec3(&self) -> Vec3 {
        Vec3::new(
            self.r as f32 / 255.0,
            self.g as f32 / 255.0,
            self.b as f32 / 255.0,
        )
    }

    /// Convert to a slice by mapping the red, green and blue component to the range `0.0..=1.0`.
    pub fn to_rgb_slice(&self) -> [f32; 3] {
        [
            self.r as f32 / 255.0,
            self.g as f32 / 255.0,
            self.b as f32 / 255.0,
        ]
    }

    pub fn from_rgb_f32(r: f32, g: f32, b: f32) -> Self {
        Self {
            r: (r * 255.0) as u8,
            g: (g * 255.0) as u8,
            b: (b * 255.0) as u8,
            ..Default::default()
        }
    }
}

impl Default for Color {
    fn default() -> Self {
        Color::WHITE
    }
}
