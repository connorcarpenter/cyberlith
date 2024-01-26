use render_api::base::{Color, CpuMaterial};

use crate::{core::*, renderer::*};

///
/// A physically-based material that renders a [Geometry] in an approximate correct physical manner based on Physically Based Rendering (PBR).
/// This material is affected by lights.
///
#[derive(Clone)]
pub struct PbrMaterial {
    pub diffuse: Color,
    pub emissive: f32,
    pub shininess: f32,
    pub render_states: RenderStates,
}

impl PbrMaterial {
    ///
    /// Constructs a new physical material from a [CpuMaterial].
    pub fn new(cpu_material: &CpuMaterial) -> Self {
        Self::new_internal(cpu_material)
    }

    /// Constructs a new opaque physical material from a [CpuMaterial].
    pub fn new_opaque(cpu_material: &CpuMaterial) -> Self {
        Self::new_internal(cpu_material)
    }

    fn new_internal(cpu_material: &CpuMaterial) -> Self {
        Self {
            diffuse: cpu_material.diffuse,
            emissive: cpu_material.emissive,
            shininess: cpu_material.shine_size,
            render_states: RenderStates {
                cull: Cull::Back,
                ..Default::default()
            },
        }
    }
}

impl FromPbrMaterial for PbrMaterial {
    fn from_cpu_material(cpu_material: &CpuMaterial) -> Self {
        Self::new(cpu_material)
    }
}

impl Material for PbrMaterial {
    fn fragment_shader(&self) -> FragmentShader {
        let output = include_str!("../../shaders/physical_material.frag").to_string();
        FragmentShader { source: output }
    }

    fn use_uniforms(&self, program: &Program, camera: &RenderCamera, lights: &[&dyn Light]) {
        if !lights.is_empty() {
            for (i, light) in lights.iter().enumerate() {
                light.use_uniforms(program, i as u32);
            }
        }
        program.use_uniform_if_required("camera_position", camera.transform.translation);

        program.use_uniform("material_color", self.diffuse);
        program.use_uniform("material_emissive", self.emissive);
        program.use_uniform("material_shininess", self.shininess);
    }

    fn render_states(&self) -> RenderStates {
        self.render_states
    }
}

impl Default for PbrMaterial {
    fn default() -> Self {
        Self {
            diffuse: Color::WHITE,
            emissive: 0.0,
            shininess: 32.0,
            render_states: RenderStates::default(),
        }
    }
}
