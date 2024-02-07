use storage::Handle;

use crate::{base::CpuTexture2D};

// Render Target
#[derive(Clone, Copy)]
pub enum RenderTarget {
    Screen,
    Image(Handle<CpuTexture2D>),
}
