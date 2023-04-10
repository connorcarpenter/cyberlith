use crate::{
    core::{Program, RenderStates},
    renderer::{FragmentShader, Light, MaterialType, RenderCamera},
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
    fn use_uniforms(&self, program: &Program, camera: &RenderCamera, lights: &[&dyn Light]);

    ///
    /// Returns the render states needed to render with this material.
    ///
    fn render_states(&self) -> RenderStates;

    ///
    /// Returns the type of material.
    ///
    fn material_type(&self) -> MaterialType;
}
