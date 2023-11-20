///
/// Describes the set of attributes provided by a [geometry] and consumed by a [Material], ie. calculated in the vertex shader and then sent to the fragment shader.
/// To use an attribute for a material, add the relevant shader code to the fragment shader source (documented for each attribute) and return this struct from [Material::fragment_shader] with the relevant attribute set to true.
///
#[derive(Clone, Copy, Debug)]
pub struct FragmentAttributes {
    /// Position in world space: `in vec3 pos;`
    pub position: bool,
}

impl FragmentAttributes {
    /// All attributes
    pub const ALL: Self = Self { position: true };
    /// No attributes
    pub const NONE: Self = Self { position: false };
}
