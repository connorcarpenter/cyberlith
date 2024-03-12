#![allow(non_upper_case_globals)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::trivially_copy_pass_by_ref)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::pedantic)] // For anyone using pedantic and a source dep, this is needed

mod version;
pub use version::Version;
mod constants;
pub use constants::*;

use core::{fmt::Debug, hash::Hash};
use std::collections::HashSet;

#[cfg(not(target_arch = "wasm32"))]
mod native;
#[cfg(not(target_arch = "wasm32"))]
pub use native::*;
#[cfg(not(target_arch = "wasm32"))]
mod gl46;

#[cfg(target_arch = "wasm32")]
#[path = "web_sys.rs"]
mod web;
#[cfg(target_arch = "wasm32")]
pub use web::*;

pub type Shader = <Context as HasContext>::Shader;
pub type Program = <Context as HasContext>::Program;
pub type Buffer = <Context as HasContext>::Buffer;
pub type VertexArray = <Context as HasContext>::VertexArray;
pub type Texture = <Context as HasContext>::Texture;
pub type Sampler = <Context as HasContext>::Sampler;
pub type Fence = <Context as HasContext>::Fence;
pub type Framebuffer = <Context as HasContext>::Framebuffer;
pub type Renderbuffer = <Context as HasContext>::Renderbuffer;
pub type Query = <Context as HasContext>::Query;
pub type UniformLocation = <Context as HasContext>::UniformLocation;
pub type TransformFeedback = <Context as HasContext>::TransformFeedback;
pub type DebugCallback = Box<dyn FnMut(u32, u32, u32, u32, &str)>;

pub struct ActiveUniform {
    pub size: i32,
    pub utype: u32,
    pub name: String,
}

pub struct ActiveAttribute {
    pub size: i32,
    pub atype: u32,
    pub name: String,
}

pub struct ActiveTransformFeedback {
    pub size: i32,
    pub tftype: u32,
    pub name: String,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct DebugMessageLogEntry {
    source: u32,
    msg_type: u32,
    id: u32,
    severity: u32,
    message: String,
}

pub enum PixelPackData<'a> {
    BufferOffset(u32),
    Slice(&'a mut [u8]),
}

pub enum PixelUnpackData<'a> {
    BufferOffset(u32),
    Slice(&'a [u8]),
}

pub enum CompressedPixelUnpackData<'a> {
    BufferRange(core::ops::Range<u32>),
    Slice(&'a [u8]),
}

// these fields are ordered according to spec: https://docs.gl/gl4/glMultiDrawArraysIndirect
pub struct DrawArraysIndirectCommand {
    // vertex count
    count: u32,
    // number of instances
    instance_count: u32,
    // first vertex index
    first: u32,
    // ?
    base_instance: u32,
}

impl DrawArraysIndirectCommand {
    pub fn new(count: u32, instance_count: u32, first: u32, base_instance: u32) -> Self {
        Self {
            count,
            instance_count,
            first,
            base_instance,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.count.to_le_bytes());
        bytes.extend_from_slice(&self.instance_count.to_le_bytes());
        bytes.extend_from_slice(&self.first.to_le_bytes());
        bytes.extend_from_slice(&self.base_instance.to_le_bytes());
        bytes
    }
}

pub trait HasContext {
    type Shader: Copy + Clone + Debug + Eq + Hash + Ord + PartialEq + PartialOrd;
    type Program: Copy + Clone + Debug + Eq + Hash + Ord + PartialEq + PartialOrd;
    type Buffer: Copy + Clone + Debug + Eq + Hash + Ord + PartialEq + PartialOrd;
    type VertexArray: Copy + Clone + Debug + Eq + Hash + Ord + PartialEq + PartialOrd;
    type Texture: Copy + Clone + Debug + Eq + Hash + Ord + PartialEq + PartialOrd;
    type Sampler: Copy + Clone + Debug + Eq + Hash + Ord + PartialEq + PartialOrd;
    type Fence: Copy + Clone + Debug + Eq + Hash + Ord + PartialEq + PartialOrd;
    type Framebuffer: Copy + Clone + Debug + Eq + Hash + Ord + PartialEq + PartialOrd;
    type Renderbuffer: Copy + Clone + Debug + Eq + Hash + Ord + PartialEq + PartialOrd;
    type Query: Copy + Clone + Debug + Eq + Hash + Ord + PartialEq + PartialOrd;
    type TransformFeedback: Copy + Clone + Debug + Eq + Hash + Ord + PartialEq + PartialOrd;
    type UniformLocation: Clone + Debug;

    fn supported_extensions(&self) -> &HashSet<String>;

    fn supports_debug(&self) -> bool;

    fn version(&self) -> &Version;

    unsafe fn create_framebuffer(&self) -> Result<Self::Framebuffer, String>;

    unsafe fn is_framebuffer(&self, framebuffer: Self::Framebuffer) -> bool;

    unsafe fn create_query(&self) -> Result<Self::Query, String>;

    unsafe fn create_renderbuffer(&self) -> Result<Self::Renderbuffer, String>;

    unsafe fn is_renderbuffer(&self, renderbuffer: Self::Renderbuffer) -> bool;

    unsafe fn create_sampler(&self) -> Result<Self::Sampler, String>;

    unsafe fn create_shader(&self, shader_type: u32) -> Result<Self::Shader, String>;

    unsafe fn is_shader(&self, shader: Self::Shader) -> bool;

    unsafe fn create_texture(&self) -> Result<Self::Texture, String>;

    unsafe fn create_named_texture(&self, target: u32) -> Result<Self::Texture, String>;

    unsafe fn is_texture(&self, texture: Self::Texture) -> bool;

    unsafe fn delete_shader(&self, shader: Self::Shader);

    unsafe fn shader_source(&self, shader: Self::Shader, source: &str);

    unsafe fn compile_shader(&self, shader: Self::Shader);

    unsafe fn get_shader_completion_status(&self, shader: Self::Shader) -> bool;

    unsafe fn get_shader_compile_status(&self, shader: Self::Shader) -> bool;

    unsafe fn get_shader_info_log(&self, shader: Self::Shader) -> String;

    unsafe fn create_program(&self) -> Result<Self::Program, String>;

    unsafe fn is_program(&self, program: Self::Program) -> bool;

    unsafe fn delete_program(&self, program: Self::Program);

    unsafe fn attach_shader(&self, program: Self::Program, shader: Self::Shader);

    unsafe fn detach_shader(&self, program: Self::Program, shader: Self::Shader);

    unsafe fn link_program(&self, program: Self::Program);

    unsafe fn get_program_completion_status(&self, program: Self::Program) -> bool;

    unsafe fn get_program_link_status(&self, program: Self::Program) -> bool;

    unsafe fn get_program_info_log(&self, program: Self::Program) -> String;

    unsafe fn get_active_uniforms(&self, program: Self::Program) -> u32;

    unsafe fn get_active_uniform(
        &self,
        program: Self::Program,
        index: u32,
    ) -> Option<ActiveUniform>;

    unsafe fn use_program(&self, program: Option<Self::Program>);

    unsafe fn create_buffer(&self) -> Result<Self::Buffer, String>;

    unsafe fn create_named_buffer(&self) -> Result<Self::Buffer, String>;

    unsafe fn is_buffer(&self, buffer: Self::Buffer) -> bool;

    unsafe fn bind_buffer(&self, target: u32, buffer: Option<Self::Buffer>);

    unsafe fn bind_buffer_base(&self, target: u32, index: u32, buffer: Option<Self::Buffer>);

    unsafe fn bind_buffer_range(
        &self,
        target: u32,
        index: u32,
        buffer: Option<Self::Buffer>,
        offset: i32,
        size: i32,
    );

    unsafe fn bind_framebuffer(&self, target: u32, framebuffer: Option<Self::Framebuffer>);

    unsafe fn bind_renderbuffer(&self, target: u32, renderbuffer: Option<Self::Renderbuffer>);

    unsafe fn blit_framebuffer(
        &self,
        src_x0: i32,
        src_y0: i32,
        src_x1: i32,
        src_y1: i32,
        dst_x0: i32,
        dst_y0: i32,
        dst_x1: i32,
        dst_y1: i32,
        mask: u32,
        filter: u32,
    );

    unsafe fn create_vertex_array(&self) -> Result<Self::VertexArray, String>;

    unsafe fn delete_vertex_array(&self, vertex_array: Self::VertexArray);

    unsafe fn bind_vertex_array(&self, vertex_array: Option<Self::VertexArray>);

    unsafe fn clear_color(&self, red: f32, green: f32, blue: f32, alpha: f32);

    unsafe fn supports_f64_precision() -> bool;

    unsafe fn clear_depth_f32(&self, depth: f32);

    unsafe fn clear_stencil(&self, stencil: i32);

    unsafe fn clear(&self, mask: u32);

    unsafe fn pixel_store_i32(&self, parameter: u32, value: i32);

    unsafe fn pixel_store_bool(&self, parameter: u32, value: bool);

    unsafe fn buffer_data_size(&self, target: u32, size: i32, usage: u32);

    unsafe fn buffer_data_u8_slice(&self, target: u32, data: &[u8], usage: u32);

    unsafe fn named_buffer_data_u8_slice(&self, buffer: Self::Buffer, data: &[u8], usage: u32);

    unsafe fn buffer_sub_data_u8_slice(&self, target: u32, offset: i32, src_data: &[u8]);

    unsafe fn get_buffer_sub_data(&self, target: u32, offset: i32, dst_data: &mut [u8]);

    unsafe fn check_framebuffer_status(&self, target: u32) -> u32;

    unsafe fn clear_buffer_i32_slice(&self, target: u32, draw_buffer: u32, values: &[i32]);

    unsafe fn clear_buffer_u32_slice(&self, target: u32, draw_buffer: u32, values: &[u32]);

    unsafe fn clear_buffer_f32_slice(&self, target: u32, draw_buffer: u32, values: &[f32]);

    unsafe fn clear_buffer_depth_stencil(
        &self,
        target: u32,
        draw_buffer: u32,
        depth: f32,
        stencil: i32,
    );

    unsafe fn client_wait_sync(&self, fence: Self::Fence, flags: u32, timeout: i32) -> u32;
    unsafe fn wait_sync(&self, fence: Self::Fence, flags: u32, timeout: u64);

    unsafe fn copy_buffer_sub_data(
        &self,
        src_target: u32,
        dst_target: u32,
        src_offset: i32,
        dst_offset: i32,
        size: i32,
    );

    unsafe fn copy_image_sub_data(
        &self,
        src_name: Self::Texture,
        src_target: u32,
        src_level: i32,
        src_x: i32,
        src_y: i32,
        src_z: i32,
        dst_name: Self::Texture,
        dst_target: u32,
        dst_level: i32,
        dst_x: i32,
        dst_y: i32,
        dst_z: i32,
        src_width: i32,
        src_height: i32,
        src_depth: i32,
    );

    unsafe fn copy_tex_image_2d(
        &self,
        target: u32,
        level: i32,
        internal_format: u32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        border: i32,
    );

    unsafe fn copy_tex_sub_image_2d(
        &self,
        target: u32,
        level: i32,
        x_offset: i32,
        y_offset: i32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    );

    unsafe fn copy_tex_sub_image_3d(
        &self,
        target: u32,
        level: i32,
        x_offset: i32,
        y_offset: i32,
        z_offset: i32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    );

    unsafe fn delete_buffer(&self, buffer: Self::Buffer);

    unsafe fn delete_framebuffer(&self, framebuffer: Self::Framebuffer);

    unsafe fn delete_query(&self, query: Self::Query);

    unsafe fn delete_renderbuffer(&self, renderbuffer: Self::Renderbuffer);

    unsafe fn delete_sampler(&self, texture: Self::Sampler);

    unsafe fn delete_sync(&self, fence: Self::Fence);

    unsafe fn delete_texture(&self, texture: Self::Texture);

    unsafe fn disable(&self, parameter: u32);

    unsafe fn disable_vertex_attrib_array(&self, index: u32);

    unsafe fn draw_arrays(&self, mode: u32, first: i32, count: i32);

    unsafe fn draw_arrays_instanced(&self, mode: u32, first: i32, count: i32, instance_count: i32);

    unsafe fn multi_draw_arrays_instanced(
        &self,
        mode: u32,
        draw_commands: Vec<DrawArraysIndirectCommand>,
    );

    unsafe fn draw_buffer(&self, buffer: u32);

    unsafe fn draw_buffers(&self, buffers: &[u32]);

    unsafe fn draw_elements(&self, mode: u32, count: i32, element_type: u32, offset: i32);

    unsafe fn draw_elements_instanced(
        &self,
        mode: u32,
        count: i32,
        element_type: u32,
        offset: i32,
        instance_count: i32,
    );

    unsafe fn enable(&self, parameter: u32);

    unsafe fn is_enabled(&self, parameter: u32) -> bool;

    unsafe fn enable_vertex_array_attrib(&self, vao: Self::VertexArray, index: u32);

    unsafe fn enable_vertex_attrib_array(&self, index: u32);

    unsafe fn flush(&self);

    unsafe fn framebuffer_renderbuffer(
        &self,
        target: u32,
        attachment: u32,
        renderbuffer_target: u32,
        renderbuffer: Option<Self::Renderbuffer>,
    );

    unsafe fn framebuffer_texture_2d(
        &self,
        target: u32,
        attachment: u32,
        texture_target: u32,
        texture: Option<Self::Texture>,
        level: i32,
    );

    unsafe fn framebuffer_texture_layer(
        &self,
        target: u32,
        attachment: u32,
        texture: Option<Self::Texture>,
        level: i32,
        layer: i32,
    );

    unsafe fn front_face(&self, value: u32);

    unsafe fn get_error(&self) -> u32;

    unsafe fn get_tex_parameter_i32(&self, target: u32, parameter: u32) -> i32;

    unsafe fn get_buffer_parameter_i32(&self, target: u32, parameter: u32) -> i32;

    unsafe fn get_parameter_i32(&self, parameter: u32) -> i32;

    unsafe fn get_parameter_i32_slice(&self, parameter: u32, out: &mut [i32]);

    unsafe fn get_parameter_f32(&self, parameter: u32) -> f32;

    unsafe fn get_parameter_f32_slice(&self, parameter: u32, out: &mut [f32]);

    unsafe fn get_parameter_indexed_i32(&self, parameter: u32, index: u32) -> i32;

    unsafe fn get_parameter_indexed_string(&self, parameter: u32, index: u32) -> String;

    unsafe fn get_parameter_string(&self, parameter: u32) -> String;

    unsafe fn get_active_uniform_block_parameter_i32(
        &self,
        program: Self::Program,
        uniform_block_index: u32,
        parameter: u32,
    ) -> i32;

    unsafe fn get_active_uniform_block_parameter_i32_slice(
        &self,
        program: Self::Program,
        uniform_block_index: u32,
        parameter: u32,
        out: &mut [i32],
    );

    unsafe fn get_active_uniform_block_name(
        &self,
        program: Self::Program,
        uniform_block_index: u32,
    ) -> String;

    unsafe fn get_uniform_location(
        &self,
        program: Self::Program,
        name: &str,
    ) -> Option<Self::UniformLocation>;

    unsafe fn get_attrib_location(&self, program: Self::Program, name: &str) -> Option<u32>;

    unsafe fn bind_attrib_location(&self, program: Self::Program, index: u32, name: &str);

    unsafe fn get_active_attributes(&self, program: Self::Program) -> u32;

    unsafe fn get_active_attribute(
        &self,
        program: Self::Program,
        index: u32,
    ) -> Option<ActiveAttribute>;

    unsafe fn get_sync_status(&self, fence: Self::Fence) -> u32;

    unsafe fn is_sync(&self, fence: Self::Fence) -> bool;

    unsafe fn renderbuffer_storage(
        &self,
        target: u32,
        internal_format: u32,
        width: i32,
        height: i32,
    );

    unsafe fn renderbuffer_storage_multisample(
        &self,
        target: u32,
        samples: i32,
        internal_format: u32,
        width: i32,
        height: i32,
    );

    unsafe fn sampler_parameter_f32(&self, sampler: Self::Sampler, name: u32, value: f32);

    unsafe fn sampler_parameter_i32(&self, sampler: Self::Sampler, name: u32, value: i32);

    unsafe fn generate_mipmap(&self, target: u32);

    unsafe fn generate_texture_mipmap(&self, texture: Self::Texture);

    unsafe fn tex_image_2d(
        &self,
        target: u32,
        level: i32,
        internal_format: i32,
        width: i32,
        height: i32,
        border: i32,
        format: u32,
        ty: u32,
        pixels: Option<&[u8]>,
    );

    unsafe fn compressed_tex_image_2d(
        &self,
        target: u32,
        level: i32,
        internal_format: i32,
        width: i32,
        height: i32,
        border: i32,
        image_size: i32,
        pixels: &[u8],
    );

    unsafe fn tex_image_3d(
        &self,
        target: u32,
        level: i32,
        internal_format: i32,
        width: i32,
        height: i32,
        depth: i32,
        border: i32,
        format: u32,
        ty: u32,
        pixels: Option<&[u8]>,
    );

    unsafe fn compressed_tex_image_3d(
        &self,
        target: u32,
        level: i32,
        internal_format: i32,
        width: i32,
        height: i32,
        depth: i32,
        border: i32,
        image_size: i32,
        pixels: &[u8],
    );

    unsafe fn tex_storage_2d(
        &self,
        target: u32,
        levels: i32,
        internal_format: u32,
        width: i32,
        height: i32,
    );

    unsafe fn tex_storage_3d(
        &self,
        target: u32,
        levels: i32,
        internal_format: u32,
        width: i32,
        height: i32,
        depth: i32,
    );

    unsafe fn texture_storage_3d(
        &self,
        texture: Self::Texture,
        levels: i32,
        internal_format: u32,
        width: i32,
        height: i32,
        depth: i32,
    );

    unsafe fn get_uniform_i32(
        &self,
        program: Self::Program,
        location: &Self::UniformLocation,
        v: &mut [i32],
    );

    unsafe fn get_uniform_f32(
        &self,
        program: Self::Program,
        location: &Self::UniformLocation,
        v: &mut [f32],
    );

    unsafe fn uniform_1_i32(&self, location: Option<&Self::UniformLocation>, x: i32);

    unsafe fn uniform_2_i32(&self, location: Option<&Self::UniformLocation>, x: i32, y: i32);

    unsafe fn uniform_3_i32(
        &self,
        location: Option<&Self::UniformLocation>,
        x: i32,
        y: i32,
        z: i32,
    );

    unsafe fn uniform_4_i32(
        &self,
        location: Option<&Self::UniformLocation>,
        x: i32,
        y: i32,
        z: i32,
        w: i32,
    );

    unsafe fn uniform_1_i32_slice(&self, location: Option<&Self::UniformLocation>, v: &[i32]);

    unsafe fn uniform_2_i32_slice(&self, location: Option<&Self::UniformLocation>, v: &[i32]);

    unsafe fn uniform_3_i32_slice(&self, location: Option<&Self::UniformLocation>, v: &[i32]);

    unsafe fn uniform_4_i32_slice(&self, location: Option<&Self::UniformLocation>, v: &[i32]);

    unsafe fn uniform_1_u32(&self, location: Option<&Self::UniformLocation>, x: u32);

    unsafe fn uniform_2_u32(&self, location: Option<&Self::UniformLocation>, x: u32, y: u32);

    unsafe fn uniform_3_u32(
        &self,
        location: Option<&Self::UniformLocation>,
        x: u32,
        y: u32,
        z: u32,
    );

    unsafe fn uniform_4_u32(
        &self,
        location: Option<&Self::UniformLocation>,
        x: u32,
        y: u32,
        z: u32,
        w: u32,
    );

    unsafe fn uniform_1_u32_slice(&self, location: Option<&Self::UniformLocation>, v: &[u32]);

    unsafe fn uniform_2_u32_slice(&self, location: Option<&Self::UniformLocation>, v: &[u32]);

    unsafe fn uniform_3_u32_slice(&self, location: Option<&Self::UniformLocation>, v: &[u32]);

    unsafe fn uniform_4_u32_slice(&self, location: Option<&Self::UniformLocation>, v: &[u32]);

    unsafe fn uniform_1_f32(&self, location: Option<&Self::UniformLocation>, x: f32);

    unsafe fn uniform_2_f32(&self, location: Option<&Self::UniformLocation>, x: f32, y: f32);

    unsafe fn uniform_3_f32(
        &self,
        location: Option<&Self::UniformLocation>,
        x: f32,
        y: f32,
        z: f32,
    );

    unsafe fn uniform_4_f32(
        &self,
        location: Option<&Self::UniformLocation>,
        x: f32,
        y: f32,
        z: f32,
        w: f32,
    );

    unsafe fn uniform_1_f32_slice(&self, location: Option<&Self::UniformLocation>, v: &[f32]);

    unsafe fn uniform_2_f32_slice(&self, location: Option<&Self::UniformLocation>, v: &[f32]);

    unsafe fn uniform_3_f32_slice(&self, location: Option<&Self::UniformLocation>, v: &[f32]);

    unsafe fn uniform_4_f32_slice(&self, location: Option<&Self::UniformLocation>, v: &[f32]);

    unsafe fn uniform_matrix_2_f32_slice(
        &self,
        location: Option<&Self::UniformLocation>,
        transpose: bool,
        v: &[f32],
    );

    unsafe fn uniform_matrix_2x3_f32_slice(
        &self,
        location: Option<&Self::UniformLocation>,
        transpose: bool,
        v: &[f32],
    );

    unsafe fn uniform_matrix_2x4_f32_slice(
        &self,
        location: Option<&Self::UniformLocation>,
        transpose: bool,
        v: &[f32],
    );

    unsafe fn uniform_matrix_3x2_f32_slice(
        &self,
        location: Option<&Self::UniformLocation>,
        transpose: bool,
        v: &[f32],
    );

    unsafe fn uniform_matrix_3_f32_slice(
        &self,
        location: Option<&Self::UniformLocation>,
        transpose: bool,
        v: &[f32],
    );

    unsafe fn uniform_matrix_3x4_f32_slice(
        &self,
        location: Option<&Self::UniformLocation>,
        transpose: bool,
        v: &[f32],
    );

    unsafe fn uniform_matrix_4x2_f32_slice(
        &self,
        location: Option<&Self::UniformLocation>,
        transpose: bool,
        v: &[f32],
    );

    unsafe fn uniform_matrix_4x3_f32_slice(
        &self,
        location: Option<&Self::UniformLocation>,
        transpose: bool,
        v: &[f32],
    );

    unsafe fn uniform_matrix_4_f32_slice(
        &self,
        location: Option<&Self::UniformLocation>,
        transpose: bool,
        v: &[f32],
    );

    unsafe fn cull_face(&self, value: u32);

    unsafe fn color_mask(&self, red: bool, green: bool, blue: bool, alpha: bool);

    unsafe fn depth_mask(&self, value: bool);

    unsafe fn blend_color(&self, red: f32, green: f32, blue: f32, alpha: f32);

    unsafe fn line_width(&self, width: f32);

    unsafe fn invalidate_framebuffer(&self, target: u32, attachments: &[u32]);

    unsafe fn polygon_offset(&self, factor: f32, units: f32);

    unsafe fn polygon_mode(&self, face: u32, mode: u32);

    unsafe fn finish(&self);

    unsafe fn bind_texture(&self, target: u32, texture: Option<Self::Texture>);

    unsafe fn bind_sampler(&self, unit: u32, sampler: Option<Self::Sampler>);

    unsafe fn active_texture(&self, unit: u32);

    unsafe fn fence_sync(&self, condition: u32, flags: u32) -> Result<Self::Fence, String>;

    unsafe fn tex_parameter_f32(&self, target: u32, parameter: u32, value: f32);

    unsafe fn tex_parameter_i32(&self, target: u32, parameter: u32, value: i32);

    unsafe fn texture_parameter_i32(&self, texture: Self::Texture, parameter: u32, value: i32);

    unsafe fn tex_sub_image_2d(
        &self,
        target: u32,
        level: i32,
        x_offset: i32,
        y_offset: i32,
        width: i32,
        height: i32,
        format: u32,
        ty: u32,
        pixels: PixelUnpackData,
    );

    unsafe fn compressed_tex_sub_image_2d(
        &self,
        target: u32,
        level: i32,
        x_offset: i32,
        y_offset: i32,
        width: i32,
        height: i32,
        format: u32,
        pixels: CompressedPixelUnpackData,
    );

    unsafe fn tex_sub_image_3d(
        &self,
        target: u32,
        level: i32,
        x_offset: i32,
        y_offset: i32,
        z_offset: i32,
        width: i32,
        height: i32,
        depth: i32,
        format: u32,
        ty: u32,
        pixels: PixelUnpackData,
    );

    unsafe fn texture_sub_image_3d(
        &self,
        texture: Self::Texture,
        level: i32,
        x_offset: i32,
        y_offset: i32,
        z_offset: i32,
        width: i32,
        height: i32,
        depth: i32,
        format: u32,
        ty: u32,
        pixels: PixelUnpackData,
    );

    unsafe fn compressed_tex_sub_image_3d(
        &self,
        target: u32,
        level: i32,
        x_offset: i32,
        y_offset: i32,
        z_offset: i32,
        width: i32,
        height: i32,
        depth: i32,
        format: u32,
        pixels: CompressedPixelUnpackData,
    );

    unsafe fn depth_func(&self, func: u32);

    unsafe fn depth_range_f32(&self, near: f32, far: f32);

    unsafe fn scissor(&self, x: i32, y: i32, width: i32, height: i32);

    unsafe fn vertex_array_attrib_binding_f32(
        &self,
        vao: Self::VertexArray,
        index: u32,
        binding_index: u32,
    );

    unsafe fn vertex_array_attrib_format_f32(
        &self,
        vao: Self::VertexArray,
        index: u32,
        size: i32,
        data_type: u32,
        normalized: bool,
        relative_offset: u32,
    );

    unsafe fn vertex_array_attrib_format_i32(
        &self,
        vao: Self::VertexArray,
        index: u32,
        size: i32,
        data_type: u32,
        relative_offset: u32,
    );

    unsafe fn vertex_array_element_buffer(
        &self,
        vao: Self::VertexArray,
        buffer: Option<Self::Buffer>,
    );

    unsafe fn vertex_array_vertex_buffer(
        &self,
        vao: Self::VertexArray,
        binding_index: u32,
        buffer: Option<Self::Buffer>,
        offset: i32,
        stride: i32,
    );

    unsafe fn vertex_attrib_divisor(&self, index: u32, divisor: u32);

    unsafe fn vertex_attrib_pointer_f32(
        &self,
        index: u32,
        size: i32,
        data_type: u32,
        normalized: bool,
        stride: i32,
        offset: i32,
    );

    unsafe fn vertex_attrib_pointer_i32(
        &self,
        index: u32,
        size: i32,
        data_type: u32,
        stride: i32,
        offset: i32,
    );

    unsafe fn vertex_attrib_1_f32(&self, index: u32, x: f32);

    unsafe fn vertex_attrib_2_f32(&self, index: u32, x: f32, y: f32);

    unsafe fn vertex_attrib_3_f32(&self, index: u32, x: f32, y: f32, z: f32);

    unsafe fn vertex_attrib_4_f32(&self, index: u32, x: f32, y: f32, z: f32, w: f32);

    unsafe fn vertex_attrib_1_f32_slice(&self, index: u32, v: &[f32]);

    unsafe fn vertex_attrib_2_f32_slice(&self, index: u32, v: &[f32]);

    unsafe fn vertex_attrib_3_f32_slice(&self, index: u32, v: &[f32]);

    unsafe fn vertex_attrib_4_f32_slice(&self, index: u32, v: &[f32]);

    unsafe fn viewport(&self, x: i32, y: i32, width: i32, height: i32);

    unsafe fn blend_equation(&self, mode: u32);

    unsafe fn blend_equation_separate(&self, mode_rgb: u32, mode_alpha: u32);

    unsafe fn blend_func(&self, src: u32, dst: u32);

    unsafe fn blend_func_separate(
        &self,
        src_rgb: u32,
        dst_rgb: u32,
        src_alpha: u32,
        dst_alpha: u32,
    );

    unsafe fn stencil_func(&self, func: u32, reference: i32, mask: u32);

    unsafe fn stencil_func_separate(&self, face: u32, func: u32, reference: i32, mask: u32);

    unsafe fn stencil_mask(&self, mask: u32);

    unsafe fn stencil_mask_separate(&self, face: u32, mask: u32);

    unsafe fn stencil_op(&self, stencil_fail: u32, depth_fail: u32, pass: u32);

    unsafe fn stencil_op_separate(&self, face: u32, stencil_fail: u32, depth_fail: u32, pass: u32);

    unsafe fn get_uniform_block_index(&self, program: Self::Program, name: &str) -> Option<u32>;

    unsafe fn uniform_block_binding(&self, program: Self::Program, index: u32, binding: u32);

    unsafe fn read_buffer(&self, src: u32);

    unsafe fn read_pixels(
        &self,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        format: u32,
        gltype: u32,
        pixels: PixelPackData,
    );

    unsafe fn begin_query(&self, target: u32, query: Self::Query);

    unsafe fn end_query(&self, target: u32);

    unsafe fn get_query_parameter_u32(&self, query: Self::Query, parameter: u32) -> u32;

    unsafe fn delete_transform_feedback(&self, transform_feedback: Self::TransformFeedback);

    unsafe fn create_transform_feedback(&self) -> Result<Self::TransformFeedback, String>;

    unsafe fn bind_transform_feedback(
        &self,
        target: u32,
        transform_feedback: Option<Self::TransformFeedback>,
    );

    unsafe fn begin_transform_feedback(&self, primitive_mode: u32);

    unsafe fn end_transform_feedback(&self);

    unsafe fn pause_transform_feedback(&self);

    unsafe fn resume_transform_feedback(&self);

    unsafe fn transform_feedback_varyings(
        &self,
        program: Self::Program,
        varyings: &[&str],
        buffer_mode: u32,
    );

    unsafe fn get_transform_feedback_varying(
        &self,
        program: Self::Program,
        index: u32,
    ) -> Option<ActiveTransformFeedback>;

    unsafe fn max_shader_compiler_threads(&self, count: u32);
}
