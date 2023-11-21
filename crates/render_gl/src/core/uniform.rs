use math::{Mat2, Mat3, Mat4, Quat, Vec2, Vec3, Vec4};
use render_api::base::Color;

use crate::core::DataType;

///
/// Possible types that can be send as a uniform to a shader (a variable that is uniformly available when processing all vertices and fragments).
///
pub trait UniformDataType: DataType {}

impl UniformDataType for u8 {}
impl UniformDataType for u16 {}
impl UniformDataType for u32 {}
impl UniformDataType for i8 {}
impl UniformDataType for i16 {}
impl UniformDataType for i32 {}
impl UniformDataType for f32 {}

impl UniformDataType for Vec2 {}
impl UniformDataType for Vec3 {}
impl UniformDataType for Vec4 {}

impl UniformDataType for Color {}
impl UniformDataType for Quat {}

impl UniformDataType for Mat2 {}
impl UniformDataType for Mat3 {}
impl UniformDataType for Mat4 {}
