use math::Vec3;
use render_api::components::Viewport;

use crate::core::{Context, DataType, Program, RenderStates, TextureDataType, VertexBuffer};

///
/// Applies a 2D/screen space effect to the given viewport. Can for example be used for adding an effect on top of a rendered image.
/// The fragment shader get the uv coordinates of the viewport (specified by `in vec2 uvs;`),
/// where uv coordinates of `(0, 0)` corresponds to the bottom left corner of the viewport and `(1, 1)` to the top right corner.
///
pub fn apply_effect(
    fragment_shader_source: &str,
    render_states: RenderStates,
    viewport: Viewport,
    use_uniforms: impl FnOnce(&Program),
) {
    let position_buffer = full_screen_buffer();
    Context::get()
        .program(
            "
            in vec3 position;
            out vec2 uvs;
            void main()
            {
                uvs = 0.5 * position.xy + 0.5;
                gl_Position = vec4(position, 1.0);
            }
        "
            .to_owned(),
            fragment_shader_source.to_owned(),
            |program| {
                use_uniforms(program);
                program.use_vertex_attribute("position", &position_buffer);
                program.draw_arrays(render_states, viewport, 3);
            },
        )
        .expect("Failed compiling shader");
}

pub fn full_screen_buffer() -> VertexBuffer {
    VertexBuffer::new_with_data(&[
        Vec3::new(-3.0, -1.0, 0.0),
        Vec3::new(3.0, -1.0, 0.0),
        Vec3::new(0.0, 2.0, 0.0),
    ])
}

pub fn to_byte_slice<T: DataType>(data: &[T]) -> &[u8] {
    unsafe {
        std::slice::from_raw_parts(
            data.as_ptr() as *const _,
            data.len() * std::mem::size_of::<T>(),
        )
    }
}

pub fn from_byte_slice<T: DataType>(data: &[u8]) -> &[T] {
    unsafe {
        let (_prefix, values, _suffix) = data.align_to::<T>();
        values
    }
}

pub fn format_from_data_type<T: DataType>() -> u32 {
    match T::size() {
        1 => gl::RED,
        2 => gl::RG,
        3 => gl::RGB,
        4 => gl::RGBA,
        _ => unreachable!(),
    }
}

pub fn flip_y<T: TextureDataType>(pixels: &mut [T], width: usize, height: usize) {
    for row in 0..height / 2 {
        for col in 0..width {
            let index0 = width * row + col;
            let index1 = width * (height - row - 1) + col;
            pixels.swap(index0, index1);
        }
    }
}
