use glow::HasContext;
use half::f16;

use render_api::base::{
    Interpolation, Texture2D as CpuTexture, TextureData, TextureDataType as ApiTextureDataType,
    Wrapping,
};

use crate::core::{ColorTarget, Context, flip_y, format_from_data_type, texture::*, to_byte_slice};

///
/// A 2D texture, basically an image that is transferred to the GPU.
///
pub struct Texture2DImpl {
    id: glow::Texture,
    width: u32,
    height: u32,
    data_byte_size: usize,
}

impl Texture2DImpl {
    ///
    /// Construcs a new texture with the given data.
    ///
    pub fn new(cpu_texture: &CpuTexture) -> Self {
        match cpu_texture.initial_data() {
            Some(TextureData::RU8(ref data)) => Self::new_with_data(cpu_texture, data),
            Some(TextureData::RgU8(ref data)) => Self::new_with_data(cpu_texture, data),
            Some(TextureData::RgbU8(ref data)) => Self::new_with_data(cpu_texture, data),
            Some(TextureData::RgbaU8(ref data)) => Self::new_with_data(cpu_texture, data),
            Some(TextureData::RF16(ref data)) => Self::new_with_data(cpu_texture, data),
            Some(TextureData::RgF16(ref data)) => Self::new_with_data(cpu_texture, data),
            Some(TextureData::RgbF16(ref data)) => Self::new_with_data(cpu_texture, data),
            Some(TextureData::RgbaF16(ref data)) => Self::new_with_data(cpu_texture, data),
            Some(TextureData::RF32(ref data)) => Self::new_with_data(cpu_texture, data),
            Some(TextureData::RgF32(ref data)) => Self::new_with_data(cpu_texture, data),
            Some(TextureData::RgbF32(ref data)) => Self::new_with_data(cpu_texture, data),
            Some(TextureData::RgbaF32(ref data)) => Self::new_with_data(cpu_texture, data),
            None => Self::new_empty_from_cpu(cpu_texture),
        }
    }

    fn new_with_data<T: TextureDataType>(cpu_texture: &CpuTexture, data: &[T]) -> Self {
        let mut texture = Self::new_empty::<T>(
            cpu_texture.width(),
            cpu_texture.height(),
            cpu_texture.min_filter(),
            cpu_texture.mag_filter(),
            cpu_texture.wrap_s(),
            cpu_texture.wrap_t(),
        );
        texture.fill(data);
        texture
    }

    fn new_empty_from_cpu(cpu_texture: &CpuTexture) -> Self {
        match cpu_texture.data_type() {
            ApiTextureDataType::RU8 => Self::new_empty_from_cpu_typed::<u8>(cpu_texture),
            ApiTextureDataType::RgU8 => Self::new_empty_from_cpu_typed::<[u8; 2]>(cpu_texture),
            ApiTextureDataType::RgbU8 => Self::new_empty_from_cpu_typed::<[u8; 3]>(cpu_texture),
            ApiTextureDataType::RgbaU8 => Self::new_empty_from_cpu_typed::<[u8; 4]>(cpu_texture),
            ApiTextureDataType::RF16 => Self::new_empty_from_cpu_typed::<f16>(cpu_texture),
            ApiTextureDataType::RgF16 => Self::new_empty_from_cpu_typed::<[f16; 2]>(cpu_texture),
            ApiTextureDataType::RgbF16 => Self::new_empty_from_cpu_typed::<[f16; 3]>(cpu_texture),
            ApiTextureDataType::RgbaF16 => Self::new_empty_from_cpu_typed::<[f16; 4]>(cpu_texture),
            ApiTextureDataType::RF32 => Self::new_empty_from_cpu_typed::<f32>(cpu_texture),
            ApiTextureDataType::RgF32 => Self::new_empty_from_cpu_typed::<[f32; 2]>(cpu_texture),
            ApiTextureDataType::RgbF32 => Self::new_empty_from_cpu_typed::<[f32; 3]>(cpu_texture),
            ApiTextureDataType::RgbaF32 => Self::new_empty_from_cpu_typed::<[f32; 4]>(cpu_texture),
        }
    }

    fn new_empty_from_cpu_typed<T: TextureDataType>(cpu_texture: &CpuTexture) -> Self {
        Self::new_empty::<T>(
            cpu_texture.width(),
            cpu_texture.height(),
            cpu_texture.min_filter(),
            cpu_texture.mag_filter(),
            cpu_texture.wrap_s(),
            cpu_texture.wrap_t(),
        )
    }

    ///
    /// Constructs a new empty 2D texture with the given parameters.
    /// The format is determined by the generic [TextureDataType] parameter
    /// (for example, if [u8; 4] is specified, the format is RGBA and the data type is byte).
    ///
    pub fn new_empty<T: TextureDataType>(
        width: u32,
        height: u32,
        min_filter: Interpolation,
        mag_filter: Interpolation,
        wrap_s: Wrapping,
        wrap_t: Wrapping,
    ) -> Self {
        let id = generate();
        let texture = Self {
            id,
            width,
            height,
            data_byte_size: std::mem::size_of::<T>(),
        };
        texture.bind();
        set_parameters(
            glow::TEXTURE_2D,
            min_filter,
            mag_filter,
            wrap_s,
            wrap_t,
            None,
        );
        unsafe {
            Context::get().tex_storage_2d(
                glow::TEXTURE_2D,
                1 as i32,
                T::internal_format(),
                width as i32,
                height as i32,
            );
        }
        texture
    }

    ///
    /// Fills this texture with the given data.
    ///
    /// # Panic
    /// Will panic if the length of the data does not correspond to the width, height and format specified at construction.
    /// It is therefore necessary to create a new texture if the texture size or format has changed.
    ///
    pub fn fill<T: TextureDataType>(&mut self, data: &[T]) {
        check_data_length::<T>(self.width, self.height, 1, self.data_byte_size, data.len());
        self.bind();
        let mut data = data.to_owned();
        flip_y(&mut data, self.width as usize, self.height as usize);
        unsafe {
            Context::get().tex_sub_image_2d(
                glow::TEXTURE_2D,
                0,
                0,
                0,
                self.width as i32,
                self.height as i32,
                format_from_data_type::<T>(),
                T::data_type(),
                glow::PixelUnpackData::Slice(to_byte_slice(&data)),
            );
        }
    }

    ///
    /// Returns a [ColorTarget] which can be used to clear, write to and read from the given mip level of this texture.
    /// Combine this together with a [DepthTarget] with [RenderTarget::new] to be able to write to both a depth and color target at the same time.
    /// If `None` is specified as the mip level, the 0 level mip level is used and mip maps are generated after a write operation if a mip map filter is specified.
    /// Otherwise, the given mip level is used and no mip maps are generated.
    ///
    /// **Note:** [DepthTest] is disabled if not also writing to a depth texture.
    ///
    pub fn as_color_target(&mut self) -> ColorTarget<'_> {
        ColorTarget::new_texture2d(self)
    }

    pub fn id(&self) -> glow::Texture {
        self.id
    }

    /// The width of this texture.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// The height of this texture.
    pub fn height(&self) -> u32 {
        self.height
    }

    pub(in crate::core) fn bind_as_color_target(&self, channel: u32, mip_level: u32) {
        unsafe {
            Context::get().framebuffer_texture_2d(
                glow::FRAMEBUFFER,
                glow::COLOR_ATTACHMENT0 + channel,
                glow::TEXTURE_2D,
                Some(self.id),
                mip_level as i32,
            );
        }
    }
    pub(in crate::core) fn bind(&self) {
        unsafe {
            Context::get().bind_texture(glow::TEXTURE_2D, Some(self.id));
        }
    }
}

impl From<&CpuTexture> for Texture2DImpl {
    fn from(value: &CpuTexture) -> Self {
        Self::new(value)
    }
}

impl Drop for Texture2DImpl {
    fn drop(&mut self) {
        unsafe {
            Context::get().delete_texture(self.id);
        }
    }
}
