use render_api::base::{Color, CpuMaterial};

use crate::{core::*, renderer::*};

///
/// A physically-based material that renders a [Geometry] in an approximate correct physical manner based on Physically Based Rendering (PBR).
/// This material is affected by lights.
///
#[derive(Clone)]
pub struct PbrMaterial {
    /// Name.
    pub name: String,
    /// Albedo base color, also called diffuse color. Assumed to be in linear color space.
    pub albedo: Color,
    /// A value in the range `[0..1]` specifying how metallic the surface is.
    pub metallic: f32,
    /// A value in the range `[0..1]` specifying how rough the surface is.
    pub roughness: f32,
    /// Render states.
    pub render_states: RenderStates,
    /// Color of light shining from an object.
    pub emissive: Color,
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
            name: cpu_material.name.clone(),
            albedo: cpu_material.albedo,
            metallic: cpu_material.metallic,
            roughness: cpu_material.roughness,
            render_states: RenderStates {
                cull: Cull::Back,
                ..Default::default()
            },
            emissive: cpu_material.emissive,
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
        let attributes = FragmentAttributes {
            position: true,
            ..FragmentAttributes::NONE
        };
        let output = include_str!("../../shaders/physical_material.frag").to_string();
        FragmentShader {
            source: output,
            attributes,
        }
    }

    fn use_uniforms(&self, program: &Program, camera: &RenderCamera, lights: &[&dyn Light]) {
        if !lights.is_empty() {
            program.use_uniform_if_required("camera_position", camera.transform.translation);
            for (i, light) in lights.iter().enumerate() {
                light.use_uniforms(program, i as u32);
            }
            program.use_uniform("metallic", self.metallic);
            program.use_uniform_if_required("roughness", self.roughness);
        }
        program.use_uniform("albedo", self.albedo);
        program.use_uniform("emissive", self.emissive);
    }

    fn render_states(&self) -> RenderStates {
        self.render_states
    }
}

impl Default for PbrMaterial {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            albedo: Color::WHITE,
            metallic: 0.0,
            roughness: 1.0,
            render_states: RenderStates::default(),
            emissive: Color::BLACK,
        }
    }
}
