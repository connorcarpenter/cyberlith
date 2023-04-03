use crate::renderer::FragmentAttributes;

/// Description of a fragment shader
#[derive(Debug, Clone)]
pub struct FragmentShader {
    /// The fragment shader source code
    pub source: String,
    /// The attributes used by this fragment shader, ie. the input from the vertex shader.
    pub attributes: FragmentAttributes,
}
