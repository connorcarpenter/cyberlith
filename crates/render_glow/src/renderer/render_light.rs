use render_api::{AmbientLight, base::{AxisAlignedBoundingBox, Camera}, Transform};

use crate::{core::{Program, ColorTexture, DepthTexture}, renderer::{BaseMesh, AmbientLightImpl, RenderAmbientLight, Geometry, Light, Material, MaterialType, Mesh, Object}};

// Render Light
#[derive(Clone, Copy)]
pub enum RenderLight<'a> {
    Wrapped(&'a dyn Light),
    Ambient(&'a AmbientLight, &'a AmbientLightImpl),
}

impl<'a> RenderLight<'a> {
    pub fn wrapped(light: &'a dyn Light) -> Self {
        Self::Wrapped(light)
    }

    pub fn ambient(light: &'a AmbientLight, light_impl: &'a AmbientLightImpl) -> Self {
        Self::Ambient(light, light_impl)
    }
}

impl<'a> Light for RenderLight<'a> {
    fn shader_source(&self, i: u32) -> String {
        match self {
            Self::Wrapped(light) => light.shader_source(i),
            Self::Ambient(light, light_impl) => {
                let render_light = RenderAmbientLight::new(light, light_impl);
                render_light.shader_source(i)
            },
        }
    }

    fn use_uniforms(&self, program: &Program, i: u32) {
        match self {
            Self::Wrapped(light) => light.use_uniforms(program, i),
            Self::Ambient(light, light_impl) => {
                let render_light = RenderAmbientLight::new(light, light_impl);
                render_light.use_uniforms(program, i)
            },
        }
    }
}