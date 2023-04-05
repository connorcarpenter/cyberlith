use std::sync::Arc;

use math::{Mat3, SquareMatrix};

use crate::core::Texture2D;

///
/// A reference to a 2D texture and a texture transformation.
///
#[derive(Clone)]
pub struct Texture2DRef {
    /// A reference to the texture.
    pub texture: Arc<Texture2D>,
    /// A transformation applied to the uv coordinates before reading a texel value at those uv coordinates.
    /// This is primarily used in relation to texture atlasing.
    pub transformation: Mat3,
}

impl std::ops::Deref for Texture2DRef {
    type Target = Texture2D;
    fn deref(&self) -> &Self::Target {
        &self.texture
    }
}

impl std::convert::From<Arc<Texture2D>> for Texture2DRef {
    fn from(texture: Arc<Texture2D>) -> Self {
        Self {
            texture,
            transformation: Mat3::identity(),
        }
    }
}
