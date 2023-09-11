use render_api::components::AmbientLightColor;

use crate::{
    core::Program,
    renderer::{AmbientLightImpl, Light, RenderAmbientLight},
};

// Render Light
#[derive(Clone, Copy)]
pub enum RenderLight<'a> {
    Wrapped(&'a dyn Light),
    Ambient(&'a AmbientLightColor, &'a AmbientLightImpl),
}

impl<'a> RenderLight<'a> {
    pub fn wrapped(light: &'a dyn Light) -> Self {
        Self::Wrapped(light)
    }

    pub fn ambient(light: &'a AmbientLightColor, light_impl: &'a AmbientLightImpl) -> Self {
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
            }
        }
    }

    fn use_uniforms(&self, program: &Program, i: u32) {
        match self {
            Self::Wrapped(light) => light.use_uniforms(program, i),
            Self::Ambient(light, light_impl) => {
                let render_light = RenderAmbientLight::new(light, light_impl);
                render_light.use_uniforms(program, i)
            }
        }
    }
}
