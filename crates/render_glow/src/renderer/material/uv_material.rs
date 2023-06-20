use render_api::base::CpuMaterial;

use crate::{core::*, renderer::*};

///
/// Render the object with colors that reflect its uv coordinates which primarily is used for debug purposes.
/// The u coordinate maps to the red channel and the v coordinate to the green channel.
///
#[derive(Default, Clone)]
pub struct UVMaterial {
    /// Render states.
    pub render_states: RenderStates,
}

impl FromPbrMaterial for UVMaterial {
    fn from_cpu_material(_cpu_material: &CpuMaterial) -> Self {
        Self::default()
    }
}

impl Material for UVMaterial {
    fn fragment_shader(&self, _lights: &[&dyn Light]) -> FragmentShader {
        FragmentShader {
            source: include_str!("shaders/uv_material.frag").to_string(),
            attributes: FragmentAttributes {
                uv: true,
                ..FragmentAttributes::NONE
            },
        }
    }

    fn use_uniforms(&self, _program: &Program, _camera: &RenderCamera, _lights: &[&dyn Light]) {}

    fn render_states(&self) -> RenderStates {
        self.render_states
    }
}
