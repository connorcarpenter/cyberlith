//!
//! Different types of buffers used for sending data (primarily geometry data) to the GPU.
//!

mod element_buffer;
mod instance_buffer;
mod uniform_buffer;
mod vertex_buffer;

pub use element_buffer::*;
pub use instance_buffer::*;
pub use uniform_buffer::*;
pub use vertex_buffer::*;

use cgmath::*;
use data_type::*;
use glow::HasContext;
use half::f16;

use crate::asset::{Color, Quat};
use crate::core::*;

/// The basic data type used for each element in a [VertexBuffer] or [InstanceBuffer].
pub trait BufferDataType: DataType {}
impl BufferDataType for u8 {}
impl BufferDataType for u16 {}
impl BufferDataType for u32 {}
impl BufferDataType for i8 {}
impl BufferDataType for i16 {}
impl BufferDataType for i32 {}
impl BufferDataType for f16 {}
impl BufferDataType for f32 {}

impl<T: BufferDataType + PrimitiveDataType> BufferDataType for Vector2<T> {}
impl<T: BufferDataType + PrimitiveDataType> BufferDataType for Vector3<T> {}
impl<T: BufferDataType + PrimitiveDataType> BufferDataType for Vector4<T> {}
impl<T: BufferDataType + PrimitiveDataType> BufferDataType for [T; 2] {}
impl<T: BufferDataType + PrimitiveDataType> BufferDataType for [T; 3] {}
impl<T: BufferDataType + PrimitiveDataType> BufferDataType for [T; 4] {}

impl BufferDataType for Color {}
impl BufferDataType for Quat {}

impl<T: BufferDataType + ?Sized> BufferDataType for &T {}

struct Buffer {
    context: Context,
    id: glow::Buffer,
    attribute_count: u32,
    data_type: u32,
    data_size: u32,
}

impl Buffer {
    pub fn new(context: &Context) -> Self {
        Self {
            context: context.clone(),
            id: unsafe { context.create_buffer().expect("Failed creating buffer") },
            attribute_count: 0,
            data_type: 0,
            data_size: 0,
        }
    }

    pub fn new_with_data<T: BufferDataType>(context: &Context, data: &[T]) -> Self {
        let mut buffer = Self::new(context);
        if !data.is_empty() {
            buffer.fill(data);
        }
        buffer
    }

    pub fn fill<T: BufferDataType>(&mut self, data: &[T]) {
        self.bind();
        unsafe {
            self.context.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                to_byte_slice(data),
                if self.attribute_count > 0 {
                    glow::DYNAMIC_DRAW
                } else {
                    glow::STATIC_DRAW
                },
            );
            self.context.bind_buffer(glow::ARRAY_BUFFER, None);
        }
        self.attribute_count = data.len() as u32;
        self.data_type = T::data_type();
        self.data_size = T::size();
    }

    pub fn attribute_count(&self) -> u32 {
        self.attribute_count
    }

    pub fn bind(&self) {
        unsafe {
            self.context.bind_buffer(glow::ARRAY_BUFFER, Some(self.id));
        }
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe {
            self.context.delete_buffer(self.id);
        }
    }
}
