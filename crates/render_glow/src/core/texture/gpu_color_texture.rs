use render_api::base::CubeSide;

use crate::core::{GpuTexture2D, GpuTexture2DArray, GpuTextureCube, Program};

///
/// A reference to some type of texture containing colors.
///
#[derive(Clone, Copy)]
#[allow(missing_docs)]
pub enum GpuColorTexture<'a> {
    /// A single 2D texture.
    Single(&'a GpuTexture2D),
    /// An array of 2D textures and a set of indices into the array.
    Array {
        texture: &'a GpuTexture2DArray,
        layers: &'a [u32],
    },
    /// A cube map texture and a set of [CubeSide]s indicating the sides to use.
    CubeMap {
        texture: &'a GpuTextureCube,
        sides: &'a [CubeSide],
    },
}

impl GpuColorTexture<'_> {
    ///
    /// Returns the width of the color texture in texels.
    ///
    pub fn width(&self) -> u32 {
        match self {
            GpuColorTexture::Single(texture) => texture.width(),
            GpuColorTexture::Array { texture, .. } => texture.width(),
            GpuColorTexture::CubeMap { texture, .. } => texture.width(),
        }
    }

    ///
    /// Returns the height of the color texture in texels.
    ///
    pub fn height(&self) -> u32 {
        match self {
            GpuColorTexture::Single(texture) => texture.height(),
            GpuColorTexture::Array { texture, .. } => texture.height(),
            GpuColorTexture::CubeMap { texture, .. } => texture.height(),
        }
    }

    ///
    /// Returns the fragment shader source for using this texture in a shader.
    ///
    pub fn fragment_shader_source(&self) -> String {
        match self {
            Self::Single(_) => "
                uniform sampler2D colorMap;
                vec4 sample_color(vec2 uv)
                {
                    return texture(colorMap, uv);
                }"
            .to_owned(),
            Self::Array { .. } => "
                uniform sampler2DArray colorMap;
                uniform int colorLayers[4];
                vec4 sample_color(vec2 uv)
                {
                    return texture(colorMap, vec3(uv, colorLayers[0]));
                }
                vec4 sample_layer(vec2 uv, int index)
                {
                    return texture(colorMap, vec3(uv, colorLayers[index]));
                }"
            .to_owned(),
            Self::CubeMap { .. } => unimplemented!(),
        }
    }

    ///
    /// Sends the uniform data needed for this texture to the fragment shader.
    ///
    pub fn use_uniforms(&self, program: &Program) {
        match self {
            Self::Single(texture) => program.use_texture("colorMap", texture),
            Self::Array { texture, layers } => {
                let mut la: [i32; 4] = [0; 4];
                layers
                    .iter()
                    .enumerate()
                    .for_each(|(i, l)| la[i] = *l as i32);
                program.use_uniform_array("colorLayers", &la);
                program.use_texture_array("colorMap", texture);
            }
            Self::CubeMap { .. } => unimplemented!(),
        }
    }

    ///
    /// The resolution of the underlying texture if there is any.
    ///
    pub fn resolution(&self) -> (u32, u32) {
        match self {
            Self::Single(texture) => (texture.width(), texture.height()),
            Self::Array { texture, .. } => (texture.width(), texture.height()),
            Self::CubeMap { texture, .. } => (texture.width(), texture.height()),
        }
    }
}
