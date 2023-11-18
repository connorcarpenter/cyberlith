use glow::HasContext;

use render_api::base::{CpuTexture2D, CpuTextureData, CpuTextureDataType};

use crate::core::{flip_y, format_from_data_type, to_byte_slice, ColorTarget, Context, TextureDataType, generate, set_parameters, check_data_length};

///
/// A 2D texture, basically an image that is transferred to the GPU.
///
#[derive(Clone)]
pub struct GpuTexture2D {
    id: glow::Texture,
    width: u32,
    height: u32,
    data_byte_size: usize,
}

impl GpuTexture2D {
    ///
    /// Construcs a new texture with the given data.
    ///
    pub fn new(cpu_texture: &CpuTexture2D) -> Self {
        match cpu_texture.initial_data() {
            Some(CpuTextureData::RU8(ref data)) => Self::new_with_data(cpu_texture, data),
            Some(CpuTextureData::RgU8(ref data)) => Self::new_with_data(cpu_texture, data),
            Some(CpuTextureData::RgbU8(ref data)) => Self::new_with_data(cpu_texture, data),
            Some(CpuTextureData::RgbaU8(ref data)) => Self::new_with_data(cpu_texture, data),
            None => Self::new_empty_from_cpu(cpu_texture),
        }
    }

    fn new_with_data<T: TextureDataType>(cpu_texture: &CpuTexture2D, data: &[T]) -> Self {
        let mut texture = Self::new_empty::<T>(
            cpu_texture.width(),
            cpu_texture.height(),
        );
        texture.fill(data);
        texture
    }

    fn new_empty_from_cpu(cpu_texture: &CpuTexture2D) -> Self {
        match cpu_texture.data_type() {
            CpuTextureDataType::RU8 => Self::new_empty_from_cpu_typed::<u8>(cpu_texture),
            CpuTextureDataType::RgU8 => Self::new_empty_from_cpu_typed::<[u8; 2]>(cpu_texture),
            CpuTextureDataType::RgbU8 => Self::new_empty_from_cpu_typed::<[u8; 3]>(cpu_texture),
            CpuTextureDataType::RgbaU8 => Self::new_empty_from_cpu_typed::<[u8; 4]>(cpu_texture),
        }
    }

    fn new_empty_from_cpu_typed<T: TextureDataType>(cpu_texture: &CpuTexture2D) -> Self {
        Self::new_empty::<T>(
            cpu_texture.width(),
            cpu_texture.height(),
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
    pub fn as_color_target(&self) -> ColorTarget<'_> {
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

impl From<&CpuTexture2D> for GpuTexture2D {
    fn from(value: &CpuTexture2D) -> Self {
        Self::new(value)
    }
}

impl Drop for GpuTexture2D {
    fn drop(&mut self) {
        unsafe {
            Context::get().delete_texture(self.id);
        }
    }
}
