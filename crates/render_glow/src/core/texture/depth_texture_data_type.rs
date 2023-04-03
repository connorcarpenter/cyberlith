use half::*;

use crate::core::DepthDataType;

/// The basic data type used for each pixel in a depth texture.
pub trait DepthTextureDataType: DepthDataType {}

/// 24 bit float which can be used as [DepthTextureDataType].
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Default, Debug)]
pub struct f24 {}

impl DepthTextureDataType for f16 {}
impl DepthTextureDataType for f24 {}
impl DepthTextureDataType for f32 {}
