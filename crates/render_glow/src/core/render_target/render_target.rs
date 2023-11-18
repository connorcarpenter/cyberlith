use glow::{Framebuffer, HasContext};

use render_api::components::Viewport;

use crate::core::*;
use crate::renderer::RenderTargetExt;

///
/// Adds additional functionality to clear, read from and write to the screen (see [RenderTarget::screen]) or a color texture and
/// a depth texture at the same time (see [RenderTarget::new]).
/// If you only want to perform an operation on either a color texture or depth texture, see [ColorTarget] and [DepthTarget] respectively.
/// A render target purely adds functionality, so it can be created each time it is needed, the actual data is saved in the textures.
///
pub struct RenderTarget<'a> {
    id: Option<Framebuffer>,
    color: Option<ColorTarget<'a>>,
    depth: Option<DepthTarget<'a>>,
    width: u32,
    height: u32,
}

impl<'a> RenderTargetExt for RenderTarget<'a> {
    /// The width of this target.
    fn width(&self) -> u32 {
        self.width
    }

    /// The height of this target.
    fn height(&self) -> u32 {
        self.height
    }

    ///
    /// Writes whatever rendered in the `render` closure into the part of this render target
    ///
    fn write(&self, render: impl FnOnce()) -> &Self {
        self.bind(glow::DRAW_FRAMEBUFFER);
        render();
        self
    }
}

impl<'a> RenderTarget<'a> {
    ///
    /// Returns the screen render target for this context.
    /// Write to this render target to draw something on the screen.
    ///
    pub fn screen(width: u32, height: u32) -> Self {
        Self {
            id: None,
            color: None,
            depth: None,
            width,
            height,
        }
    }

    ///
    /// Constructs a new render target that enables rendering into the given [ColorTarget] and [DepthTarget].
    ///
    pub fn new(color: ColorTarget<'a>, depth: DepthTarget<'a>) -> Self {
        let width = color.width();
        let height = color.height();
        Self {
            id: Some(new_framebuffer()),
            color: Some(color),
            depth: Some(depth),
            width,
            height,
        }
    }

    ///
    /// Clears the color and depth of the part of this render target
    ///
    pub fn clear(&self, clear_state: ClearState) -> &Self {
        self.bind(glow::DRAW_FRAMEBUFFER);
        clear_state.apply();
        self
    }

    ///
    /// Returns the colors of the pixels in this render target.
    /// The number of channels per pixel and the data format for each channel is specified by the generic parameter.
    ///
    /// **Note:** On web, the data format needs to match the data format of the color texture.
    ///
    pub fn read_color<T: TextureDataType>(&self) -> Vec<T> {
        if self.id.is_some() && self.color.is_none() {
            panic!("cannot read color from a render target without a color target");
        }
        self.bind(glow::DRAW_FRAMEBUFFER);
        self.bind(glow::READ_FRAMEBUFFER);
        let mut data_size = std::mem::size_of::<T>();
        // On web, the format needs to be RGBA if the data type is byte.
        if data_size / T::size() as usize == 1 {
            data_size *= 4 / T::size() as usize
        }
        let mut bytes = vec![0u8; self.width as usize * self.height as usize * data_size];
        unsafe {
            Context::get().read_pixels(
                0,
                0,
                self.width as i32,
                self.height as i32,
                format_from_data_type::<T>(),
                T::data_type(),
                glow::PixelPackData::Slice(&mut bytes),
            );
        }
        let mut pixels = from_byte_slice(&bytes).to_vec();
        flip_y(&mut pixels, self.width as usize, self.height as usize);
        pixels
    }

    ///
    /// Returns the depth values in this render target.
    ///
    #[cfg(not(target_arch = "wasm32"))]
    pub fn read_depth(&self) -> Vec<f32> {
        if self.id.is_some() && self.depth.is_none() {
            panic!("cannot read depth from a render target without a depth target");
        }
        self.bind(glow::DRAW_FRAMEBUFFER);
        self.bind(glow::READ_FRAMEBUFFER);
        let mut pixels = vec![0u8; self.width as usize * self.height as usize * 4];
        unsafe {
            Context::get().read_pixels(
                0,
                0,
                self.width as i32,
                self.height as i32,
                glow::DEPTH_COMPONENT,
                glow::FLOAT,
                glow::PixelPackData::Slice(&mut pixels),
            );
        }
        from_byte_slice(&pixels).to_vec()
    }

    ///
    /// Copies the content of the color and depth texture as limited by the [WriteMask]
    /// to the part of this render target specified by the [Viewport].
    ///
    pub fn copy_from(
        &self,
        color_texture: GpuTexture2D,
        depth_texture: GpuDepthTexture2D,
        viewport: Viewport,
        write_mask: WriteMask,
    ) -> &Self {
        self.write(|| {
            let fragment_shader_source = format!(
                "{}\n{}\n
                in vec2 uvs;
                layout (location = 0) out vec4 color;
                void main()
                {{
                    color = sample_color(uvs);
                    gl_FragDepth = sample_depth(uvs);
                }}",
                color_texture.fragment_shader_source(),
                depth_texture.fragment_shader_source()
            );
            apply_effect(
                &fragment_shader_source,
                RenderStates {
                    depth_test: DepthTest::Always,
                    write_mask,
                    ..Default::default()
                },
                viewport,
                |program| {
                    color_texture.use_uniforms(program);
                    depth_texture.use_uniforms(program);
                },
            )
        })
    }

    ///
    /// Copies the content of the color texture as limited by the [WriteMask]
    /// to the part of this render target specified by the [Viewport].
    ///
    pub fn copy_from_color(
        &self,
        color_texture: GpuTexture2D,
        viewport: Viewport,
        write_mask: WriteMask,
    ) -> &Self {
        self.write(|| {
            let fragment_shader_source = format!(
                "{}\nin vec2 uvs;
                layout (location = 0) out vec4 color;
                void main()
                {{
                    color = sample_color(uvs);
                }}",
                color_texture.fragment_shader_source()
            );
            apply_effect(
                &fragment_shader_source,
                RenderStates {
                    depth_test: DepthTest::Always,
                    write_mask,
                    ..Default::default()
                },
                viewport,
                |program| {
                    color_texture.use_uniforms(program);
                },
            )
        })
    }

    ///
    /// Copies the content of the depth texture
    /// to the part of this render target specified by the [Viewport].
    ///
    pub fn copy_from_depth(&self, depth_texture: GpuDepthTexture2D, viewport: Viewport) -> &Self {
        self.write(|| {
            let fragment_shader_source = format!(
                "{}\n
                    in vec2 uvs;
                    void main()
                    {{
                        gl_FragDepth = sample_depth(uvs);
                    }}",
                depth_texture.fragment_shader_source(),
            );
            apply_effect(
                &fragment_shader_source,
                RenderStates {
                    depth_test: DepthTest::Always,
                    write_mask: WriteMask::DEPTH,
                    ..Default::default()
                },
                viewport,
                |program| {
                    depth_texture.use_uniforms(program);
                },
            )
        })
    }

    ///
    /// Creates a [RenderTarget] with the given low-level [Framebuffer]. Should only be used if the [Framebuffer] is used for something else, ie. to be able
    /// to combine this crate with functionality of another crate. Also see [Self::into_framebuffer].
    ///
    pub fn from_framebuffer(width: u32, height: u32, framebuffer: Framebuffer) -> Self {
        Self {
            id: Some(framebuffer),
            color: None,
            depth: None,
            width,
            height,
        }
    }

    ///
    /// Transforms this [RenderTarget] into a low-level [Framebuffer]. Should only be used if the [Framebuffer] is used for something else, ie. to be able
    /// to combine this crate with functionality of another crate. Also see [Self::from_framebuffer].
    ///
    pub fn into_framebuffer(mut self) -> Option<Framebuffer> {
        self.id.take()
    }

    pub(crate) fn new_color(color: ColorTarget<'a>) -> Self {
        let width = color.width();
        let height = color.height();
        Self {
            id: Some(new_framebuffer()),
            color: Some(color),
            depth: None,
            width,
            height,
        }
    }

    pub(crate) fn new_depth(depth: DepthTarget<'a>) -> Self {
        let width = depth.width();
        let height = depth.height();
        Self {
            id: Some(new_framebuffer()),
            depth: Some(depth),
            color: None,
            width,
            height,
        }
    }

    fn bind(&self, target: u32) {
        unsafe {
            Context::get().bind_framebuffer(target, self.id);
        }
        if let Some(ref color) = self.color {
            color.bind();
        }
        if let Some(ref depth) = self.depth {
            depth.bind();
        }
    }
}

impl Drop for RenderTarget<'_> {
    fn drop(&mut self) {
        unsafe {
            if let Some(id) = self.id {
                Context::get().delete_framebuffer(id);
            }
        }
    }
}

fn new_framebuffer() -> glow::Framebuffer {
    unsafe {
        Context::get()
            .create_framebuffer()
            .expect("Failed creating frame buffer")
    }
}
