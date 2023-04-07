use std::sync::Arc;

use math::Mat3;

use crate::core::Texture2DImpl;

///
/// A reference to a 2D texture and a texture transformation.
///
#[derive(Clone)]
pub struct Texture2DRef {
    /// A reference to the texture.
    pub texture: Arc<Texture2DImpl>,
    /// A transformation applied to the uv coordinates before reading a texel value at those uv coordinates.
    /// This is primarily used in relation to texture atlasing.
    pub transformation: Mat3,
}

impl std::ops::Deref for Texture2DRef {
    type Target = Texture2DImpl;
    fn deref(&self) -> &Self::Target {
        &self.texture
    }
}

impl std::convert::From<Arc<Texture2DImpl>> for Texture2DRef {
    fn from(texture: Arc<Texture2DImpl>) -> Self {
        Self {
            texture,
            transformation: Mat3::IDENTITY,
        }
    }
}
