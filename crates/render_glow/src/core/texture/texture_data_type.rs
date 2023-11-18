
use math::{Quat, Vec2, Vec3, Vec4};
use render_api::base::Color;

use crate::core::{DataType, PrimitiveDataType};

/// The basic data type used for each channel of each pixel in a texture.
pub trait TextureDataType: DataType {}
impl TextureDataType for u8 {}
impl TextureDataType for f32 {}

impl TextureDataType for Vec2 {}
impl TextureDataType for Vec3 {}
impl TextureDataType for Vec4 {}
impl<T: TextureDataType + PrimitiveDataType> TextureDataType for [T; 2] {}
impl<T: TextureDataType + PrimitiveDataType> TextureDataType for [T; 3] {}
impl<T: TextureDataType + PrimitiveDataType> TextureDataType for [T; 4] {}

impl TextureDataType for Color {}
impl TextureDataType for Quat {}
