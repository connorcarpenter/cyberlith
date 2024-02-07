use storage::AssetHash;

use crate::{base::CpuTextureDataType};
use super::CpuTextureData;

///
/// A CPU-side version of a 2D texture.
///
#[derive(Clone, Debug, PartialEq, Hash)]
pub struct CpuTexture2D {
    /// Name of this texture.
    name: String,
    /// The pixel data for the image
    initial_data: Option<CpuTextureData>,
    /// The pixel data type
    data_type: CpuTextureDataType,
    /// The width of the image
    width: u32,
    /// The height of the image
    height: u32,
}

impl AssetHash<CpuTexture2D> for CpuTexture2D {}

impl CpuTexture2D {
    pub fn from_size(width: u32, height: u32) -> Self {
        let mut output = Self::default();
        output.width = width;
        output.height = height;
        output
    }

    pub fn initial_data(&self) -> Option<&CpuTextureData> {
        self.initial_data.as_ref()
    }

    pub fn data_type(&self) -> &CpuTextureDataType {
        &self.data_type
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }
}

impl Default for CpuTexture2D {
    fn default() -> Self {
        Self {
            name: "default".to_owned(),
            initial_data: None,
            data_type: CpuTextureDataType::RgbaU8,
            width: 1,
            height: 1,
        }
    }
}
