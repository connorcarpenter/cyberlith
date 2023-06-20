use crate::base::TextureDataType;

use super::{Interpolation, TextureData, Wrapping};

///
/// A CPU-side version of a 2D texture.
///
#[derive(Clone, Debug, PartialEq)]
pub struct CpuTexture2D {
    /// Name of this texture.
    name: String,
    /// The pixel data for the image
    initial_data: Option<TextureData>,
    /// The pixel data type
    data_type: TextureDataType,
    /// The width of the image
    width: u32,
    /// The height of the image
    height: u32,
    /// The way the pixel data is interpolated when the texture is far away
    min_filter: Interpolation,
    /// The way the pixel data is interpolated when the texture is close
    mag_filter: Interpolation,
    /// Determines how the texture is sampled outside the [0..1] s coordinate range (the first value of the uv coordinates).
    wrap_s: Wrapping,
    /// Determines how the texture is sampled outside the [0..1] t coordinate range (the second value of the uv coordinates).
    wrap_t: Wrapping,
}

impl CpuTexture2D {
    pub fn from_size(width: u32, height: u32) -> Self {
        let mut output = Self::default();
        output.width = width;
        output.height = height;
        output
    }

    pub fn initial_data(&self) -> Option<&TextureData> {
        self.initial_data.as_ref()
    }

    pub fn data_type(&self) -> &TextureDataType {
        &self.data_type
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn min_filter(&self) -> Interpolation {
        self.min_filter
    }

    pub fn mag_filter(&self) -> Interpolation {
        self.mag_filter
    }

    pub fn wrap_s(&self) -> Wrapping {
        self.wrap_s
    }

    pub fn wrap_t(&self) -> Wrapping {
        self.wrap_t
    }
}

impl Default for CpuTexture2D {
    fn default() -> Self {
        Self {
            name: "default".to_owned(),
            initial_data: None,
            data_type: TextureDataType::RgbaU8,
            width: 1,
            height: 1,
            min_filter: Interpolation::Linear,
            mag_filter: Interpolation::Linear,
            wrap_s: Wrapping::Repeat,
            wrap_t: Wrapping::Repeat,
        }
    }
}
