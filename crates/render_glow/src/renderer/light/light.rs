use crate::core::*;

pub trait Light {
    fn shader_source(&self, i: u32) -> String;
    fn use_uniforms(&self, program: &Program, i: u32);
}
