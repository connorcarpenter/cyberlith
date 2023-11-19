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
    /// If the input contains an [CpuMaterial::occlusion_metallic_roughness_texture], this texture is used for both
    /// [PbrMaterial::metallic_roughness_texture] and [PbrMaterial::occlusion_texture] while any [CpuMaterial::metallic_roughness_texture] or [CpuMaterial::occlusion_texture] are ignored.
    /// Tries to infer whether this material is transparent or opaque from the alpha value of the albedo color and the alpha values in the albedo texture.
    /// Since this is not always correct, it is preferred to use [PbrMaterial::new_opaque] or [PbrMaterial::new_transparent].
    ///
    pub fn new(cpu_material: &CpuMaterial) -> Self {
        Self::new_internal(cpu_material, super::is_transparent(cpu_material))
    }

    /// Constructs a new opaque physical material from a [CpuMaterial].
    /// If the input contains an [CpuMaterial::occlusion_metallic_roughness_texture], this texture is used for both
    /// [PbrMaterial::metallic_roughness_texture] and [PbrMaterial::occlusion_texture] while any [CpuMaterial::metallic_roughness_texture] or [CpuMaterial::occlusion_texture] are ignored.
    pub fn new_opaque(cpu_material: &CpuMaterial) -> Self {
        Self::new_internal(cpu_material, false)
    }

    /// Constructs a new transparent physical material from a [CpuMaterial].
    /// If the input contains an [CpuMaterial::occlusion_metallic_roughness_texture], this texture is used for both
    /// [PbrMaterial::metallic_roughness_texture] and [PbrMaterial::occlusion_texture] while any [CpuMaterial::metallic_roughness_texture] or [CpuMaterial::occlusion_texture] are ignored.
    pub fn new_transparent(cpu_material: &CpuMaterial) -> Self {
        Self::new_internal(cpu_material, true)
    }

    fn new_internal(cpu_material: &CpuMaterial, is_transparent: bool) -> Self {
        Self {
            name: cpu_material.name.clone(),
            albedo: cpu_material.albedo,
            metallic: cpu_material.metallic,
            roughness: cpu_material.roughness,
            render_states: if is_transparent {
                RenderStates {
                    write_mask: WriteMask::COLOR,
                    blend: Blend::TRANSPARENCY,
                    ..Default::default()
                }
            } else {
                RenderStates {
                    cull: Cull::Back,
                    ..Default::default()
                }
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
    fn fragment_shader(&self, lights: &[&dyn Light]) -> FragmentShader {
        let attributes = FragmentAttributes {
            position: true,
            ..FragmentAttributes::NONE
        };
        let mut output = lights_shader_source(lights);
        output.push_str(include_str!("shaders/physical_material.frag"));
        FragmentShader {
            source: output,
            attributes,
        }
    }

    fn use_uniforms(&self, program: &Program, camera: &RenderCamera, lights: &[&dyn Light]) {
        if !lights.is_empty() {
            program.use_uniform_if_required("cameraPosition", camera.transform.translation);
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
