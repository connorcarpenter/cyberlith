use render_api::base::Camera;

use crate::{
    core::{Program, RenderStates},
    renderer::{FragmentShader, Light, MaterialType},
};

///
/// Represents a material that, together with a [geometry], can be rendered using [Geometry::render_with_material].
/// Alternatively, a geometry and a material can be combined in a [Gm],
/// thereby creating an [Object] which can be used in a render call, for example [RenderTarget::render].
///
pub trait Material: Send + Sync {
    ///
    /// Returns a [FragmentShader], ie. the fragment shader source for this material
    /// and a [FragmentAttributes] struct that describes which fragment attributes are required for rendering with this material.
    ///
    fn fragment_shader(&self, lights: &[&dyn Light]) -> FragmentShader;

    ///
    /// Sends the uniform data needed for this material to the fragment shader.
    ///
    fn use_uniforms(&self, program: &Program, camera: &Camera, lights: &[&dyn Light]);

    ///
    /// Returns the render states needed to render with this material.
    ///
    fn render_states(&self) -> RenderStates;

    ///
    /// Returns the type of material.
    ///
    fn material_type(&self) -> MaterialType;
}

impl<T: Material + ?Sized> Material for &T {
    fn fragment_shader(&self, lights: &[&dyn Light]) -> FragmentShader {
        (*self).fragment_shader(lights)
    }
    fn use_uniforms(&self, program: &Program, camera: &Camera, lights: &[&dyn Light]) {
        (*self).use_uniforms(program, camera, lights)
    }
    fn render_states(&self) -> RenderStates {
        (*self).render_states()
    }
    fn material_type(&self) -> MaterialType {
        (*self).material_type()
    }
}

impl<T: Material + ?Sized> Material for &mut T {
    fn fragment_shader(&self, lights: &[&dyn Light]) -> FragmentShader {
        (**self).fragment_shader(lights)
    }
    fn use_uniforms(&self, program: &Program, camera: &Camera, lights: &[&dyn Light]) {
        (**self).use_uniforms(program, camera, lights)
    }
    fn render_states(&self) -> RenderStates {
        (**self).render_states()
    }
    fn material_type(&self) -> MaterialType {
        (**self).material_type()
    }
}

impl<T: Material> Material for Box<T> {
    fn fragment_shader(&self, lights: &[&dyn Light]) -> FragmentShader {
        self.as_ref().fragment_shader(lights)
    }
    fn use_uniforms(&self, program: &Program, camera: &Camera, lights: &[&dyn Light]) {
        self.as_ref().use_uniforms(program, camera, lights)
    }
    fn render_states(&self) -> RenderStates {
        self.as_ref().render_states()
    }
    fn material_type(&self) -> MaterialType {
        self.as_ref().material_type()
    }
}

impl<T: Material> Material for std::sync::Arc<T> {
    fn fragment_shader(&self, lights: &[&dyn Light]) -> FragmentShader {
        self.as_ref().fragment_shader(lights)
    }
    fn use_uniforms(&self, program: &Program, camera: &Camera, lights: &[&dyn Light]) {
        self.as_ref().use_uniforms(program, camera, lights)
    }
    fn render_states(&self) -> RenderStates {
        self.as_ref().render_states()
    }
    fn material_type(&self) -> MaterialType {
        self.as_ref().material_type()
    }
}

impl<T: Material> Material for std::sync::RwLock<T> {
    fn fragment_shader(&self, lights: &[&dyn Light]) -> FragmentShader {
        self.read().unwrap().fragment_shader(lights)
    }
    fn use_uniforms(&self, program: &Program, camera: &Camera, lights: &[&dyn Light]) {
        self.read().unwrap().use_uniforms(program, camera, lights)
    }
    fn render_states(&self) -> RenderStates {
        self.read().unwrap().render_states()
    }
    fn material_type(&self) -> MaterialType {
        self.read().unwrap().material_type()
    }
}
