
use crate::core::DepthDataType;

/// The basic data type used for each pixel in a depth texture.
pub trait DepthTextureDataType: DepthDataType {}

impl DepthTextureDataType for f32 {}
