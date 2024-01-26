use crate::{base::CpuTexture2D, Handle};

// Render Target
#[derive(Clone, Copy)]
pub enum RenderTarget {
    Screen,
    Image(Handle<CpuTexture2D>),
}
