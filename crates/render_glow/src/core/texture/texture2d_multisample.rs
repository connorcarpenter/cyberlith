use glow::HasContext;

use crate::core::texture::*;

pub struct Texture2DMultisample {
    context: Context,
    id: glow::Renderbuffer,
    width: u32,
    height: u32,
    number_of_samples: u32,
}

impl Texture2DMultisample {
    pub fn new<T: TextureDataType>(
        context: &Context,
        width: u32,
        height: u32,
        number_of_samples: u32,
    ) -> Self {
        let id = unsafe {
            context
                .create_renderbuffer()
                .expect("Failed creating render buffer")
        };
        let texture = Self {
            context: context.clone(),
            id,
            width,
            height,
            number_of_samples,
        };
        texture.bind();
        unsafe {
            context.renderbuffer_storage_multisample(
                glow::RENDERBUFFER,
                number_of_samples as i32,
                T::internal_format(),
                width as i32,
                height as i32,
            );
        }
        texture
    }

    /// The width of this texture.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// The height of this texture.
    pub fn height(&self) -> u32 {
        self.height
    }

    /// The number of samples for each fragment.
    pub fn number_of_samples(&self) -> u32 {
        self.number_of_samples
    }

    pub(in crate::core) fn bind_as_color_target(&self, channel: u32) {
        unsafe {
            self.context.framebuffer_renderbuffer(
                glow::FRAMEBUFFER,
                glow::COLOR_ATTACHMENT0 + channel,
                glow::RENDERBUFFER,
                Some(self.id),
            );
        }
    }
    pub(in crate::core) fn bind(&self) {
        unsafe {
            self.context
                .bind_renderbuffer(glow::RENDERBUFFER, Some(self.id));
        }
    }
}

impl Drop for Texture2DMultisample {
    fn drop(&mut self) {
        unsafe {
            self.context.delete_renderbuffer(self.id);
        }
    }
}
