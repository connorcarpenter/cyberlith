use crate::{base::CpuTexture2D, Handle};

// Render Target
pub enum RenderTarget {
    Screen,
    Image(Handle<CpuTexture2D>),
}
