use crate::core::*;

/// Represents a light source.
pub trait Light {
    /// The fragment shader source for calculating this lights contribution to the color in a fragment.
    /// It should contain a function with this signature
    /// `vec3 calculate_lighting{}(vec3 surface_color, vec3 position, vec3 normal, vec3 view_direction, float metallic, float roughness, float occlusion)`
    /// Where `{}` is replaced with the number i given as input.
    /// This function should return the color contribution for this light on the surface with the given surface parameters.
    fn shader_source(&self, i: u32) -> String;
    /// Should bind the uniforms that is needed for calculating this lights contribution to the color in [Light::shader_source].
    fn use_uniforms(&self, program: &Program, i: u32);
}

impl<T: Light + ?Sized> Light for &T {
    fn shader_source(&self, i: u32) -> String {
        (*self).shader_source(i)
    }
    fn use_uniforms(&self, program: &Program, i: u32) {
        (*self).use_uniforms(program, i)
    }
}

impl<T: Light + ?Sized> Light for &mut T {
    fn shader_source(&self, i: u32) -> String {
        (**self).shader_source(i)
    }
    fn use_uniforms(&self, program: &Program, i: u32) {
        (**self).use_uniforms(program, i)
    }
}

impl<T: Light> Light for Box<T> {
    fn shader_source(&self, i: u32) -> String {
        self.as_ref().shader_source(i)
    }
    fn use_uniforms(&self, program: &Program, i: u32) {
        self.as_ref().use_uniforms(program, i)
    }
}

impl<T: Light> Light for std::sync::Arc<T> {
    fn shader_source(&self, i: u32) -> String {
        self.as_ref().shader_source(i)
    }
    fn use_uniforms(&self, program: &Program, i: u32) {
        self.as_ref().use_uniforms(program, i)
    }
}

impl<T: Light> Light for std::sync::Arc<std::sync::RwLock<T>> {
    fn shader_source(&self, i: u32) -> String {
        self.read().unwrap().shader_source(i)
    }
    fn use_uniforms(&self, program: &Program, i: u32) {
        self.read().unwrap().use_uniforms(program, i)
    }
}
