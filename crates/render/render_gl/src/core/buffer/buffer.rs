use gl::{Buffer as GlBuffer, HasContext};

use crate::core::{to_byte_slice, BufferDataType, Context};

pub struct Buffer {
    id: GlBuffer,
    attribute_count: u32,
    pub data_type: u32,
    pub data_size: u32,
}

impl Buffer {
    pub fn new() -> Self {
        Self {
            id: unsafe {
                Context::get()
                    .create_buffer()
                    .expect("Failed creating buffer")
            },
            attribute_count: 0,
            data_type: 0,
            data_size: 0,
        }
    }

    // this is used actually
    pub fn new_with_data<T: BufferDataType>(data: &[T]) -> Self {
        let mut buffer = Self::new();
        if !data.is_empty() {
            buffer.fill(data);
        }
        buffer
    }

    pub fn fill<T: BufferDataType>(&mut self, data: &[T]) {
        self.bind();
        unsafe {
            let context = Context::get();
            context.buffer_data_u8_slice(
                gl::ARRAY_BUFFER,
                to_byte_slice(data),
                if self.attribute_count > 0 {
                    gl::DYNAMIC_DRAW
                } else {
                    gl::STATIC_DRAW
                },
            );
            context.bind_buffer(gl::ARRAY_BUFFER, None);
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
            Context::get().bind_buffer(gl::ARRAY_BUFFER, Some(self.id));
        }
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe {
            Context::get().delete_buffer(self.id);
        }
    }
}
