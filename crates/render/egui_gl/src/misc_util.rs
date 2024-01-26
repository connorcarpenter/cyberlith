#![allow(unsafe_code)]

use gl::HasContext as _;

pub(crate) unsafe fn compile_shader(
    gl: &gl::Context,
    shader_type: u32,
    source: &str,
) -> Result<gl::Shader, String> {
    let shader = gl.create_shader(shader_type)?;

    gl.shader_source(shader, source);

    gl.compile_shader(shader);

    if gl.get_shader_compile_status(shader) {
        Ok(shader)
    } else {
        Err(gl.get_shader_info_log(shader))
    }
}

pub(crate) unsafe fn link_program<'a, T: IntoIterator<Item = &'a gl::Shader>>(
    gl: &gl::Context,
    shaders: T,
) -> Result<gl::Program, String> {
    let program = gl.create_program()?;

    for shader in shaders {
        gl.attach_shader(program, *shader);
    }

    gl.link_program(program);

    if gl.get_program_link_status(program) {
        Ok(program)
    } else {
        Err(gl.get_program_info_log(program))
    }
}
