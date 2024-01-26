use std::hash::{Hash, Hasher};

///
/// The pixel/texel data for a [Texture2D] or [Texture3D].
///
/// If 2D data, the data array should start with the top left texel and then one row at a time.
/// The indices `(row, column)` into the 2D data would look like
/// ```notrust
/// [
/// (0, 0), (1, 0), .., // First row
/// (0, 1), (1, 1), .., // Second row
/// ..
/// ]
/// ```
/// If 3D data, the data array would look like the 2D data, one layer/image at a time.
/// The indices `(row, column, layer)` into the 3D data would look like
/// ```notrust
/// [
/// (0, 0, 0), (1, 0, 0), .., // First row in first layer
/// (0, 1, 0), (1, 1, 0), .., // Second row in first layer
/// ..
/// (0, 0, 1), (1, 0, 1), .., // First row in second layer
/// (0, 1, 1), (1, 1, 1), ..,  // Second row in second layer
/// ..
/// ]
/// ```
///
#[derive(Clone, PartialEq)]
pub enum CpuTextureData {
    /// One byte in the red channel.
    RU8(Vec<u8>),
    /// One byte in the red and green channel.
    RgU8(Vec<[u8; 2]>),
    /// One byte in the red, green and blue channel.
    RgbU8(Vec<[u8; 3]>),
    /// One byte in the red, green, blue and alpha channel.
    RgbaU8(Vec<[u8; 4]>),
}

impl Hash for CpuTextureData {
    fn hash<H: Hasher>(&self, _state: &mut H) {
        match self {
            _ => {
                todo!()
            }
        }
    }
}

impl std::fmt::Debug for CpuTextureData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RU8(values) => write!(f, "R u8 ({:?})", values.len()),
            Self::RgU8(values) => write!(f, "RG u8 ({:?})", values.len()),
            Self::RgbU8(values) => write!(f, "RGB u8 ({:?})", values.len()),
            Self::RgbaU8(values) => write!(f, "RGBA u8 ({:?})", values.len()),
        }
    }
}

#[derive(Hash, Clone, Debug, PartialEq)]
pub enum CpuTextureDataType {
    /// One byte in the red channel.
    RU8,
    /// One byte in the red and green channel.
    RgU8,
    /// One byte in the red, green and blue channel.
    RgbU8,
    /// One byte in the red, green, blue and alpha channel.
    RgbaU8,
}
