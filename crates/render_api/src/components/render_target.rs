use crate::{base::Texture2D, Handle};

// Render Target
pub enum RenderTarget {
    Screen,
    Image(Handle<Texture2D>),
}