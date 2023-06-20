use glow::HasContext;

use render_api::base::{CpuTexture3D, CpuTextureData, Interpolation, Wrapping};

use crate::core::{Context, format_from_data_type, texture::*, to_byte_slice};

///
/// A 3D color texture.
///
pub struct GpuTexture3D {
    id: glow::Texture,
    width: u32,
    height: u32,
    depth: u32,
    data_byte_size: usize,
}

impl GpuTexture3D {
    ///
    /// Construcs a new 3D texture with the given data.
    ///
    pub fn new(cpu_texture: &CpuTexture3D) -> Self {
        match cpu_texture.data {
            CpuTextureData::RU8(ref data) => Self::new_with_data(cpu_texture, data),
            CpuTextureData::RgU8(ref data) => Self::new_with_data(cpu_texture, data),
            CpuTextureData::RgbU8(ref data) => Self::new_with_data(cpu_texture, data),
            CpuTextureData::RgbaU8(ref data) => Self::new_with_data(cpu_texture, data),
            CpuTextureData::RF16(ref data) => Self::new_with_data(cpu_texture, data),
            CpuTextureData::RgF16(ref data) => Self::new_with_data(cpu_texture, data),
            CpuTextureData::RgbF16(ref data) => Self::new_with_data(cpu_texture, data),
            CpuTextureData::RgbaF16(ref data) => Self::new_with_data(cpu_texture, data),
            CpuTextureData::RF32(ref data) => Self::new_with_data(cpu_texture, data),
            CpuTextureData::RgF32(ref data) => Self::new_with_data(cpu_texture, data),
            CpuTextureData::RgbF32(ref data) => Self::new_with_data(cpu_texture, data),
            CpuTextureData::RgbaF32(ref data) => Self::new_with_data(cpu_texture, data),
        }
    }

    fn new_with_data<T: TextureDataType>(cpu_texture: &CpuTexture3D, data: &[T]) -> Self {
        let mut texture = Self::new_empty::<T>(
            cpu_texture.width,
            cpu_texture.height,
            cpu_texture.depth,
            cpu_texture.min_filter,
            cpu_texture.mag_filter,
            cpu_texture.wrap_s,
            cpu_texture.wrap_t,
            cpu_texture.wrap_r,
        );
        texture.fill(data);
        texture
    }

    ///
    /// Creates a new empty 3D color texture.
    ///
    pub fn new_empty<T: TextureDataType>(
        width: u32,
        height: u32,
        depth: u32,
        min_filter: Interpolation,
        mag_filter: Interpolation,
        wrap_s: Wrapping,
        wrap_t: Wrapping,
        wrap_r: Wrapping,
    ) -> Self {
        let id = generate();
        let texture = Self {
            id,
            width,
            height,
            depth,
            data_byte_size: std::mem::size_of::<T>(),
        };
        texture.bind();
        set_parameters(
            glow::TEXTURE_3D,
            min_filter,
            mag_filter,
            wrap_s,
            wrap_t,
            Some(wrap_r),
        );
        unsafe {
            Context::get().tex_storage_3d(
                glow::TEXTURE_3D,
                1,
                T::internal_format(),
                width as i32,
                height as i32,
                depth as i32,
            );
        }
        texture
    }

    ///
    /// Fills this texture with the given data.
    ///
    /// # Panic
    /// Will panic if the length of the data does not correspond to the width, height, depth and format specified at construction.
    /// It is therefore necessary to create a new texture if the texture size or format has changed.
    ///
    pub fn fill<T: TextureDataType>(&mut self, data: &[T]) {
        check_data_length::<T>(
            self.width,
            self.height,
            self.depth,
            self.data_byte_size,
            data.len(),
        );
        self.bind();
        unsafe {
            Context::get().tex_sub_image_3d(
                glow::TEXTURE_3D,
                0,
                0,
                0,
                0,
                self.width as i32,
                self.height as i32,
                self.depth as i32,
                format_from_data_type::<T>(),
                T::data_type(),
                glow::PixelUnpackData::Slice(to_byte_slice(data)),
            );
        }
    }

    /// The width of this texture.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// The height of this texture.
    pub fn height(&self) -> u32 {
        self.height
    }

    /// The depth of this texture.
    pub fn depth(&self) -> u32 {
        self.depth
    }

    pub(in crate::core) fn bind(&self) {
        unsafe {
            Context::get().bind_texture(glow::TEXTURE_3D, Some(self.id));
        }
    }
}

impl Drop for GpuTexture3D {
    fn drop(&mut self) {
        unsafe {
            Context::get().delete_texture(self.id);
        }
    }
}
