use glow::HasContext;
use half::f16;

use render_api::base::{
    Interpolation, Texture2D as CpuTexture, TextureData, TextureDataType as ApiTextureDataType,
    Wrapping,
};

use crate::core::{ColorTarget, Context, flip_y, format_from_data_type, texture::*, to_byte_slice};

///
/// A array of 2D color textures that can be rendered into.
///
/// **Note:** [DepthTest] is disabled if not also writing to a [DepthTarget].
/// Use a [RenderTarget] to write to both color and depth.
///
pub struct Texture2DArray {
    id: glow::Texture,
    width: u32,
    height: u32,
    depth: u32,
    data_byte_size: usize,
}

impl Texture2DArray {
    ///
    /// Creates a new texture array from the given [Texture2DImpl]s.
    /// All of the cpu textures must contain data with the same [TextureDataType] and the same width and height.
    ///
    pub fn new(cpu_textures: &[&CpuTexture]) -> Self {
        let cpu_texture = cpu_textures
            .get(0)
            .expect("Expect at least one texture in a texture array");
        match &cpu_texture.initial_data() {
            Some(TextureData::RU8(_)) => Self::new_with_data(
                cpu_texture,
                &cpu_textures.iter().map(|t| ru8_data(t)).collect::<Vec<_>>(),
            ),
            Some(TextureData::RgU8(_)) => Self::new_with_data(
                cpu_texture,
                &cpu_textures
                    .iter()
                    .map(|t| rgu8_data(t))
                    .collect::<Vec<_>>(),
            ),
            Some(TextureData::RgbU8(_)) => Self::new_with_data(
                cpu_texture,
                &cpu_textures
                    .iter()
                    .map(|t| rgbu8_data(t))
                    .collect::<Vec<_>>(),
            ),
            Some(TextureData::RgbaU8(_)) => Self::new_with_data(
                cpu_texture,
                &cpu_textures
                    .iter()
                    .map(|t| rgbau8_data(t))
                    .collect::<Vec<_>>(),
            ),
            Some(TextureData::RF16(_)) => Self::new_with_data(
                cpu_texture,
                &cpu_textures
                    .iter()
                    .map(|t| rf16_data(t))
                    .collect::<Vec<_>>(),
            ),
            Some(TextureData::RgF16(_)) => Self::new_with_data(
                cpu_texture,
                &cpu_textures
                    .iter()
                    .map(|t| rgf16_data(t))
                    .collect::<Vec<_>>(),
            ),
            Some(TextureData::RgbF16(_)) => Self::new_with_data(
                cpu_texture,
                &cpu_textures
                    .iter()
                    .map(|t| rgbf16_data(t))
                    .collect::<Vec<_>>(),
            ),
            Some(TextureData::RgbaF16(_)) => Self::new_with_data(
                cpu_texture,
                &cpu_textures
                    .iter()
                    .map(|t| rgbaf16_data(t))
                    .collect::<Vec<_>>(),
            ),
            Some(TextureData::RF32(_)) => Self::new_with_data(
                cpu_texture,
                &cpu_textures
                    .iter()
                    .map(|t| rf32_data(t))
                    .collect::<Vec<_>>(),
            ),
            Some(TextureData::RgF32(_)) => Self::new_with_data(
                cpu_texture,
                &cpu_textures
                    .iter()
                    .map(|t| rgf32_data(t))
                    .collect::<Vec<_>>(),
            ),
            Some(TextureData::RgbF32(_)) => Self::new_with_data(
                cpu_texture,
                &cpu_textures
                    .iter()
                    .map(|t| rgbf32_data(t))
                    .collect::<Vec<_>>(),
            ),
            Some(TextureData::RgbaF32(_)) => Self::new_with_data(
                cpu_texture,
                &cpu_textures
                    .iter()
                    .map(|t| rgbaf32_data(t))
                    .collect::<Vec<_>>(),
            ),
            _ => Self::new_empty_from_cpu(cpu_texture, cpu_textures.len() as u32),
        }
    }

    fn new_with_data<T: TextureDataType>(cpu_texture: &CpuTexture, data: &[&[T]]) -> Self {
        let mut texture = Self::new_empty::<T>(
            cpu_texture.width(),
            cpu_texture.height(),
            data.len() as u32,
            cpu_texture.min_filter(),
            cpu_texture.mag_filter(),
            cpu_texture.wrap_s(),
            cpu_texture.wrap_t(),
        );
        texture.fill(data);
        texture
    }

    fn new_empty_from_cpu(cpu_texture: &CpuTexture, depth: u32) -> Self {
        match cpu_texture.data_type() {
            ApiTextureDataType::RU8 => Self::new_empty_from_cpu_typed::<u8>(cpu_texture, depth),
            ApiTextureDataType::RgU8 => {
                Self::new_empty_from_cpu_typed::<[u8; 2]>(cpu_texture, depth)
            }
            ApiTextureDataType::RgbU8 => {
                Self::new_empty_from_cpu_typed::<[u8; 3]>(cpu_texture, depth)
            }
            ApiTextureDataType::RgbaU8 => {
                Self::new_empty_from_cpu_typed::<[u8; 4]>(cpu_texture, depth)
            }
            ApiTextureDataType::RF16 => Self::new_empty_from_cpu_typed::<f16>(cpu_texture, depth),
            ApiTextureDataType::RgF16 => {
                Self::new_empty_from_cpu_typed::<[f16; 2]>(cpu_texture, depth)
            }
            ApiTextureDataType::RgbF16 => {
                Self::new_empty_from_cpu_typed::<[f16; 3]>(cpu_texture, depth)
            }
            ApiTextureDataType::RgbaF16 => {
                Self::new_empty_from_cpu_typed::<[f16; 4]>(cpu_texture, depth)
            }
            ApiTextureDataType::RF32 => Self::new_empty_from_cpu_typed::<f32>(cpu_texture, depth),
            ApiTextureDataType::RgF32 => {
                Self::new_empty_from_cpu_typed::<[f32; 2]>(cpu_texture, depth)
            }
            ApiTextureDataType::RgbF32 => {
                Self::new_empty_from_cpu_typed::<[f32; 3]>(cpu_texture, depth)
            }
            ApiTextureDataType::RgbaF32 => {
                Self::new_empty_from_cpu_typed::<[f32; 4]>(cpu_texture, depth)
            }
        }
    }

    fn new_empty_from_cpu_typed<T: TextureDataType>(cpu_texture: &CpuTexture, depth: u32) -> Self {
        Self::new_empty::<T>(
            cpu_texture.width(),
            cpu_texture.height(),
            depth,
            cpu_texture.min_filter(),
            cpu_texture.mag_filter(),
            cpu_texture.wrap_s(),
            cpu_texture.wrap_t(),
        )
    }

    ///
    /// Creates a new array of 2D textures.
    ///
    pub fn new_empty<T: TextureDataType>(
        width: u32,
        height: u32,
        depth: u32,
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
            depth,
            data_byte_size: std::mem::size_of::<T>(),
        };
        texture.bind();
        set_parameters(
            glow::TEXTURE_2D_ARRAY,
            min_filter,
            mag_filter,
            wrap_s,
            wrap_t,
            None,
        );
        unsafe {
            Context::get().tex_storage_3d(
                glow::TEXTURE_2D_ARRAY,
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
    /// Fills the texture array with the given pixel data.
    ///
    /// # Panic
    /// Will panic if the data does not correspond to the width, height, depth and format specified at construction.
    /// It is therefore necessary to create a new texture if the texture size or format has changed.
    ///
    pub fn fill<T: TextureDataType>(&mut self, data: &[&[T]]) {
        for (i, data) in data.iter().enumerate() {
            self.fill_layer(i as u32, data);
        }
    }

    ///
    /// Fills the given layer in the texture array with the given pixel data.
    ///
    /// # Panic
    /// Will panic if the layer number is bigger than the number of layers or if the data does not correspond to the width, height and format specified at construction.
    /// It is therefore necessary to create a new texture if the texture size or format has changed.
    ///
    pub fn fill_layer<T: TextureDataType>(&mut self, layer: u32, data: &[T]) {
        if layer >= self.depth {
            panic!(
                "cannot fill the layer {} with data, since there are only {} layers in the texture array",
                layer, self.depth
            )
        }
        check_data_length::<T>(self.width, self.height, 1, self.data_byte_size, data.len());
        self.bind();
        let mut data = (*data).to_owned();
        flip_y(&mut data, self.width as usize, self.height as usize);
        unsafe {
            Context::get().tex_sub_image_3d(
                glow::TEXTURE_2D_ARRAY,
                0,
                0,
                0,
                layer as i32,
                self.width as i32,
                self.height as i32,
                1,
                format_from_data_type::<T>(),
                T::data_type(),
                glow::PixelUnpackData::Slice(to_byte_slice(&data)),
            );
        }
    }

    ///
    /// Returns a [ColorTarget] which can be used to clear, write to and read from the given layers and mip level of this texture.
    /// Combine this together with a [DepthTarget] with [RenderTarget::new] to be able to write to both a depth and color target at the same time.
    /// If `None` is specified as the mip level, the 0 level mip level is used and mip maps are generated after a write operation if a mip map filter is specified.
    /// Otherwise, the given mip level is used and no mip maps are generated.
    ///
    /// **Note:** [DepthTest] is disabled if not also writing to a depth texture.
    ///
    pub fn as_color_target<'a>(&'a mut self, layers: &'a [u32]) -> ColorTarget<'a> {
        ColorTarget::new_texture_2d_array(self, layers)
    }

    /// The width of this texture.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// The height of this texture.
    pub fn height(&self) -> u32 {
        self.height
    }

    /// The number of layers.
    pub fn depth(&self) -> u32 {
        self.depth
    }

    pub(in crate::core) fn bind_as_color_target(&self, layer: u32, channel: u32, mip_level: u32) {
        unsafe {
            Context::get().framebuffer_texture_layer(
                glow::DRAW_FRAMEBUFFER,
                glow::COLOR_ATTACHMENT0 + channel,
                Some(self.id),
                mip_level as i32,
                layer as i32,
            );
        }
    }

    pub(in crate::core) fn bind(&self) {
        unsafe {
            Context::get().bind_texture(glow::TEXTURE_2D_ARRAY, Some(self.id));
        }
    }
}

impl Drop for Texture2DArray {
    fn drop(&mut self) {
        unsafe {
            Context::get().delete_texture(self.id);
        }
    }
}
