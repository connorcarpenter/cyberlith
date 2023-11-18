
use math::*;
use render_api::base::Color;

use crate::core::*;

/// The basic data type used for each element in a [VertexBuffer] or [InstanceBuffer].
pub trait BufferDataType: DataType {}
impl BufferDataType for u8 {}
impl BufferDataType for u16 {}
impl BufferDataType for u32 {}
impl BufferDataType for i8 {}
impl BufferDataType for i16 {}
impl BufferDataType for i32 {}
impl BufferDataType for f32 {}

impl BufferDataType for Vec2 {}
impl BufferDataType for Vec3 {}
impl BufferDataType for Vec4 {}
impl<T: BufferDataType + PrimitiveDataType> BufferDataType for [T; 2] {}
impl<T: BufferDataType + PrimitiveDataType> BufferDataType for [T; 3] {}
impl<T: BufferDataType + PrimitiveDataType> BufferDataType for [T; 4] {}

impl BufferDataType for Color {}
impl BufferDataType for Quat {}
