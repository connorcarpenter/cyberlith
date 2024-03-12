use render_api::components::TypedLight;

use crate::core::Program;

pub trait Light {
    fn shader_source(&self, i: u32) -> String;
    fn use_uniforms(&self, program: &Program, i: u32);
}

impl Light for TypedLight {
    fn shader_source(&self, i: u32) -> String {
        match self {
            TypedLight::Ambient(light) => light.shader_source(i),
            TypedLight::Directional(light) => light.shader_source(i),
            TypedLight::Point(light) => light.shader_source(i),
        }
    }

    fn use_uniforms(&self, program: &Program, i: u32) {
        match self {
            TypedLight::Ambient(light) => light.use_uniforms(program, i),
            TypedLight::Directional(light) => light.use_uniforms(program, i),
            TypedLight::Point(light) => light.use_uniforms(program, i),
        }
    }
}
