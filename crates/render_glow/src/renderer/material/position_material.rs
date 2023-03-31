use crate::core::*;
use crate::renderer::*;
use render_api::base::PbrMaterial;

///
/// Render the object with colors that reflect its position which primarily is used for debug purposes.
/// The x coordinate maps to the red channel, y to green and z to blue.
///
#[derive(Default, Clone)]
pub struct PositionMaterial {
    /// Render states.
    pub render_states: RenderStates,
}

impl FromPbrMaterial for PositionMaterial {
    fn from_cpu_material(_cpu_material: &PbrMaterial) -> Self {
        Self::default()
    }
}

impl Material for PositionMaterial {
    fn fragment_shader(&self, _lights: &[&dyn Light]) -> FragmentShader {
        FragmentShader {
            source: include_str!("shaders/position_material.frag").to_string(),
            attributes: FragmentAttributes {
                position: true,
                ..FragmentAttributes::NONE
            },
        }
    }

    fn use_uniforms(&self, _program: &Program, _camera: &Camera, _lights: &[&dyn Light]) {}

    fn render_states(&self) -> RenderStates {
        self.render_states
    }

    fn material_type(&self) -> MaterialType {
        MaterialType::Opaque
    }
}
