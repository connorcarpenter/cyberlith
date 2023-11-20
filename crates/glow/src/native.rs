use std::{
    collections::HashSet,
    ffi::{CStr, CString},
    num::NonZeroU32,
};

use super::*;
use crate::{gl46 as native_gl, version::Version};

#[derive(Default)]
struct Constants {
    max_label_length: i32,
}

/// Store a boxed callback (i.e., `Box<Box<dyn FnMut(...)>>`) as a raw pointer, so that it can be
/// referenced by the C API and later converted back into a `Box` and dropped.
///
/// We use a raw pointer here because `Box` aliasing rules are not fully defined, so we can'
/// guarantee that it's not undefined behavior to keep a `Box` here while it's used as a raw
/// pointer in the C API.
struct DebugCallbackRawPtr {
    callback: *mut std::os::raw::c_void,
}

impl Drop for DebugCallbackRawPtr {
    fn drop(&mut self) {
        unsafe {
            // Convert callback back into `Box` and drop it.
            let thin_ptr = Box::from_raw(self.callback as *mut DebugCallback);
            let callback = *thin_ptr;
            drop(callback);
        }
    }
}

pub struct Context {
    raw: native_gl::GlFns,
    extensions: HashSet<String>,
    constants: Constants,
    version: Version,
}

impl Context {
    pub unsafe fn from_loader_function_cstr<F>(mut loader_function: F) -> Self
    where
        F: FnMut(&CStr) -> *const std::os::raw::c_void,
    {
        let raw: native_gl::GlFns =
            native_gl::GlFns::load_with(|p: *const std::os::raw::c_char| {
                let c_str = std::ffi::CStr::from_ptr(p);
                loader_function(c_str) as *mut std::os::raw::c_void
            });

        // Retrieve and parse `GL_VERSION`
        let raw_string = raw.GetString(VERSION);

        if raw_string.is_null() {
            panic!("Reading GL_VERSION failed. Make sure there is a valid GL context currently active.")
        }

        let raw_version = std::ffi::CStr::from_ptr(raw_string as *const native_gl::GLchar)
            .to_str()
            .unwrap()
            .to_owned();
        let version = Version::parse(&raw_version).unwrap();

        // Setup extensions and constants after the context has been built
        let mut context = Self {
            raw,
            extensions: HashSet::new(),
            constants: Constants::default(),
            version,
        };

        // Use core-only functions to populate extension list
        if (context.version >= Version::new(3, 0, None, String::from("")))
            || (context.version >= Version::new_embedded(3, 0, String::from("")))
        {
            let num_extensions = context.get_parameter_i32(NUM_EXTENSIONS);
            for i in 0..num_extensions {
                let extension_name = context.get_parameter_indexed_string(EXTENSIONS, i as u32);
                context.extensions.insert(extension_name);
            }
        } else {
            // Fallback
            context.extensions.extend(
                context
                    .get_parameter_string(EXTENSIONS)
                    .split(' ')
                    .map(|s| s.to_string()),
            );
        };

        // After the extensions are known, we can populate constants (including
        // constants that depend on extensions being enabled)
        context.constants.max_label_length = if context.supports_debug() {
            context.get_parameter_i32(MAX_LABEL_LENGTH)
        } else {
            0
        };

        context
    }

    pub unsafe fn from_loader_function<F>(mut loader_function: F) -> Self
    where
        F: FnMut(&str) -> *const std::os::raw::c_void,
    {
        Self::from_loader_function_cstr(move |name| loader_function(name.to_str().unwrap()))
    }

    /// Creates a texture from an external GL name.
    ///
    /// This can be useful when a texture is created outside of glow (e.g. OpenXR surface) but glow
    /// still needs access to it for rendering.
    #[deprecated = "Use the NativeTexture constructor instead"]
    pub unsafe fn create_texture_from_gl_name(gl_name: native_gl::GLuint) -> NativeTexture {
        NativeTexture(non_zero_gl_name(gl_name))
    }

    /// Creates a framebuffer from an external GL name.
    ///
    /// This can be useful when a framebuffer is created outside of glow (e.g: via `surfman` or another
    /// crate that supports sharing of buffers between GL contexts), but glow needs to set it as a target.
    #[deprecated = "Use the NativeFramebuffer constructor instead"]
    pub unsafe fn create_framebuffer_from_gl_name(gl_name: native_gl::GLuint) -> NativeFramebuffer {
        NativeFramebuffer(non_zero_gl_name(gl_name))
    }
}

impl std::fmt::Debug for Context {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Native_GL_Context")
    }
}

fn non_zero_gl_name(value: native_gl::GLuint) -> NonZeroU32 {
    NonZeroU32::new(value as u32).expect("expected non-zero GL name")
}

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NativeShader(pub NonZeroU32);

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NativeProgram(pub NonZeroU32);

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NativeBuffer(pub NonZeroU32);

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NativeVertexArray(pub NonZeroU32);

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NativeTexture(pub NonZeroU32);

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NativeSampler(pub NonZeroU32);

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NativeFence(pub native_gl::GLsync);

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NativeFramebuffer(pub NonZeroU32);

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NativeRenderbuffer(pub NonZeroU32);

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NativeQuery(pub NonZeroU32);

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NativeUniformLocation(pub native_gl::GLuint);

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NativeTransformFeedback(pub NonZeroU32);

impl HasContext for Context {
    type Shader = NativeShader;
    type Program = NativeProgram;
    type Buffer = NativeBuffer;
    type VertexArray = NativeVertexArray;
    type Texture = NativeTexture;
    type Sampler = NativeSampler;
    type Fence = NativeFence;
    type Framebuffer = NativeFramebuffer;
    type Renderbuffer = NativeRenderbuffer;
    type Query = NativeQuery;
    type UniformLocation = NativeUniformLocation;
    type TransformFeedback = NativeTransformFeedback;

    fn supported_extensions(&self) -> &HashSet<String> {
        &self.extensions
    }

    fn supports_debug(&self) -> bool {
        if self.extensions.contains("GL_KHR_debug") {
            // Supports extension (either GL or GL ES)
            true
        } else if self.version.is_embedded {
            // GL ES >= 3.2
            self.version.major == 3 && self.version.minor >= 2
        } else {
            // GL >= 4.3
            self.version.major == 4 && self.version.minor >= 3
        }
    }

    fn version(&self) -> &Version {
        &self.version
    }

    unsafe fn create_framebuffer(&self) -> Result<Self::Framebuffer, String> {
        let gl = &self.raw;
        let mut name = 0;
        gl.GenFramebuffers(1, &mut name);
        Ok(NativeFramebuffer(non_zero_gl_name(name)))
    }

    unsafe fn is_framebuffer(&self, framebuffer: Self::Framebuffer) -> bool {
        let gl = &self.raw;
        gl.IsFramebuffer(framebuffer.0.get()) != 0
    }

    unsafe fn create_query(&self) -> Result<Self::Query, String> {
        let gl = &self.raw;
        let mut name = 0;
        gl.GenQueries(1, &mut name);
        Ok(NativeQuery(non_zero_gl_name(name)))
    }

    unsafe fn create_renderbuffer(&self) -> Result<Self::Renderbuffer, String> {
        let gl = &self.raw;
        let mut name = 0;
        gl.GenRenderbuffers(1, &mut name);
        Ok(NativeRenderbuffer(non_zero_gl_name(name)))
    }

    unsafe fn is_renderbuffer(&self, renderbuffer: Self::Renderbuffer) -> bool {
        let gl = &self.raw;
        gl.IsRenderbuffer(renderbuffer.0.get()) != 0
    }

    unsafe fn create_sampler(&self) -> Result<Self::Sampler, String> {
        let gl = &self.raw;
        let mut name = 0;
        gl.GenSamplers(1, &mut name);
        Ok(NativeSampler(non_zero_gl_name(name)))
    }

    unsafe fn create_shader(&self, shader_type: u32) -> Result<Self::Shader, String> {
        let gl = &self.raw;
        Ok(NativeShader(non_zero_gl_name(
            gl.CreateShader(shader_type as u32),
        )))
    }

    unsafe fn is_shader(&self, shader: Self::Shader) -> bool {
        let gl = &self.raw;
        gl.IsShader(shader.0.get()) != 0
    }

    unsafe fn create_texture(&self) -> Result<Self::Texture, String> {
        let gl = &self.raw;
        let mut name = 0;
        gl.GenTextures(1, &mut name);
        Ok(NativeTexture(non_zero_gl_name(name)))
    }

    unsafe fn create_named_texture(&self, target: u32) -> Result<Self::Texture, String> {
        let gl = &self.raw;
        let mut name = 0;
        gl.CreateTextures(target, 1, &mut name);
        Ok(NativeTexture(non_zero_gl_name(name)))
    }

    unsafe fn is_texture(&self, texture: Self::Texture) -> bool {
        let gl = &self.raw;
        gl.IsTexture(texture.0.get()) != 0
    }

    unsafe fn delete_shader(&self, shader: Self::Shader) {
        let gl = &self.raw;
        gl.DeleteShader(shader.0.get());
    }

    unsafe fn shader_source(&self, shader: Self::Shader, source: &str) {
        let gl = &self.raw;
        gl.ShaderSource(
            shader.0.get(),
            1,
            &(source.as_ptr() as *const native_gl::GLchar),
            &(source.len() as native_gl::GLint),
        );
    }

    unsafe fn compile_shader(&self, shader: Self::Shader) {
        let gl = &self.raw;
        gl.CompileShader(shader.0.get());
    }

    unsafe fn get_shader_completion_status(&self, shader: Self::Shader) -> bool {
        let gl = &self.raw;
        let mut status = 0;
        gl.GetShaderiv(shader.0.get(), COMPLETION_STATUS, &mut status);
        1 == status
    }

    unsafe fn get_shader_compile_status(&self, shader: Self::Shader) -> bool {
        let gl = &self.raw;
        let mut status = 0;
        gl.GetShaderiv(shader.0.get(), COMPILE_STATUS, &mut status);
        1 == status
    }

    unsafe fn get_shader_info_log(&self, shader: Self::Shader) -> String {
        let gl = &self.raw;
        let mut length = 0;
        gl.GetShaderiv(shader.0.get(), INFO_LOG_LENGTH, &mut length);
        if length > 0 {
            let mut log = String::with_capacity(length as usize);
            log.extend(std::iter::repeat('\0').take(length as usize));
            gl.GetShaderInfoLog(
                shader.0.get(),
                length,
                &mut length,
                (&log[..]).as_ptr() as *mut native_gl::GLchar,
            );
            log.truncate(length as usize);
            log
        } else {
            String::from("")
        }
    }

    unsafe fn create_program(&self) -> Result<Self::Program, String> {
        let gl = &self.raw;
        Ok(NativeProgram(non_zero_gl_name(gl.CreateProgram())))
    }

    unsafe fn is_program(&self, program: Self::Program) -> bool {
        let gl = &self.raw;
        gl.IsProgram(program.0.get()) != 0
    }

    unsafe fn delete_program(&self, program: Self::Program) {
        let gl = &self.raw;
        gl.DeleteProgram(program.0.get());
    }

    unsafe fn attach_shader(&self, program: Self::Program, shader: Self::Shader) {
        let gl = &self.raw;
        gl.AttachShader(program.0.get(), shader.0.get());
    }

    unsafe fn detach_shader(&self, program: Self::Program, shader: Self::Shader) {
        let gl = &self.raw;
        gl.DetachShader(program.0.get(), shader.0.get());
    }

    unsafe fn link_program(&self, program: Self::Program) {
        let gl = &self.raw;
        gl.LinkProgram(program.0.get());
    }

    unsafe fn get_program_completion_status(&self, program: Self::Program) -> bool {
        let gl = &self.raw;
        let mut status = 0;
        gl.GetProgramiv(program.0.get(), COMPLETION_STATUS, &mut status);
        1 == status
    }

    unsafe fn get_program_link_status(&self, program: Self::Program) -> bool {
        let gl = &self.raw;
        let mut status = 0;
        gl.GetProgramiv(program.0.get(), LINK_STATUS, &mut status);
        1 == status
    }

    unsafe fn get_program_info_log(&self, program: Self::Program) -> String {
        let gl = &self.raw;
        let mut length = 0;
        gl.GetProgramiv(program.0.get(), INFO_LOG_LENGTH, &mut length);
        if length > 0 {
            let mut log = String::with_capacity(length as usize);
            log.extend(std::iter::repeat('\0').take(length as usize));
            gl.GetProgramInfoLog(
                program.0.get(),
                length,
                &mut length,
                (&log[..]).as_ptr() as *mut native_gl::GLchar,
            );
            log.truncate(length as usize);
            log
        } else {
            String::from("")
        }
    }

    unsafe fn get_active_uniforms(&self, program: Self::Program) -> u32 {
        let gl = &self.raw;
        let mut count = 0;
        gl.GetProgramiv(program.0.get(), ACTIVE_UNIFORMS, &mut count);
        count as u32
    }

    unsafe fn get_active_uniform(
        &self,
        program: Self::Program,
        index: u32,
    ) -> Option<ActiveUniform> {
        let gl = &self.raw;
        let mut uniform_max_size = 0;
        gl.GetProgramiv(
            program.0.get(),
            ACTIVE_UNIFORM_MAX_LENGTH,
            &mut uniform_max_size,
        );

        let mut name = String::with_capacity(uniform_max_size as usize);
        name.extend(std::iter::repeat('\0').take(uniform_max_size as usize));
        let mut length = 0;
        let mut size = 0;
        let mut utype = 0;
        gl.GetActiveUniform(
            program.0.get(),
            index,
            uniform_max_size,
            &mut length,
            &mut size,
            &mut utype,
            name.as_ptr() as *mut native_gl::GLchar,
        );
        name.truncate(length as usize);

        Some(ActiveUniform { size, utype, name })
    }

    unsafe fn use_program(&self, program: Option<Self::Program>) {
        let gl = &self.raw;
        gl.UseProgram(program.map(|p| p.0.get()).unwrap_or(0));
    }

    unsafe fn create_buffer(&self) -> Result<Self::Buffer, String> {
        let gl = &self.raw;
        let mut buffer = 0;
        gl.GenBuffers(1, &mut buffer);
        Ok(NativeBuffer(non_zero_gl_name(buffer)))
    }

    unsafe fn create_named_buffer(&self) -> Result<Self::Buffer, String> {
        let gl = &self.raw;
        let mut buffer = 0;
        gl.CreateBuffers(1, &mut buffer);
        Ok(NativeBuffer(non_zero_gl_name(buffer)))
    }

    unsafe fn is_buffer(&self, buffer: Self::Buffer) -> bool {
        let gl = &self.raw;
        gl.IsBuffer(buffer.0.get()) != 0
    }

    unsafe fn bind_buffer(&self, target: u32, buffer: Option<Self::Buffer>) {
        let gl = &self.raw;
        gl.BindBuffer(target, buffer.map(|b| b.0.get()).unwrap_or(0));
    }

    unsafe fn bind_buffer_base(&self, target: u32, index: u32, buffer: Option<Self::Buffer>) {
        let gl = &self.raw;
        gl.BindBufferBase(target, index, buffer.map(|b| b.0.get()).unwrap_or(0));
    }

    unsafe fn bind_buffer_range(
        &self,
        target: u32,
        index: u32,
        buffer: Option<Self::Buffer>,
        offset: i32,
        size: i32,
    ) {
        let gl = &self.raw;
        gl.BindBufferRange(
            target,
            index,
            buffer.map(|b| b.0.get()).unwrap_or(0),
            offset as isize,
            size as isize,
        );
    }

    unsafe fn bind_framebuffer(&self, target: u32, framebuffer: Option<Self::Framebuffer>) {
        let gl = &self.raw;
        gl.BindFramebuffer(target, framebuffer.map(|fb| fb.0.get()).unwrap_or(0));
    }

    unsafe fn bind_renderbuffer(&self, target: u32, renderbuffer: Option<Self::Renderbuffer>) {
        let gl = &self.raw;
        gl.BindRenderbuffer(target, renderbuffer.map(|rb| rb.0.get()).unwrap_or(0));
    }

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
    ) {
        let gl = &self.raw;
        gl.BlitFramebuffer(
            src_x0, src_y0, src_x1, src_y1, dst_x0, dst_y0, dst_x1, dst_y1, mask, filter,
        );
    }

    unsafe fn create_vertex_array(&self) -> Result<Self::VertexArray, String> {
        let gl = &self.raw;
        let mut vertex_array = 0;
        gl.GenVertexArrays(1, &mut vertex_array);
        Ok(NativeVertexArray(non_zero_gl_name(vertex_array)))
    }

    unsafe fn delete_vertex_array(&self, vertex_array: Self::VertexArray) {
        let gl = &self.raw;
        gl.DeleteVertexArrays(1, &vertex_array.0.get());
    }

    unsafe fn bind_vertex_array(&self, vertex_array: Option<Self::VertexArray>) {
        let gl = &self.raw;
        gl.BindVertexArray(vertex_array.map(|va| va.0.get()).unwrap_or(0));
    }

    unsafe fn clear_color(&self, red: f32, green: f32, blue: f32, alpha: f32) {
        let gl = &self.raw;
        gl.ClearColor(red, green, blue, alpha);
    }

    unsafe fn supports_f64_precision() -> bool {
        // TODO: Handle OpenGL ES
        true
    }

    unsafe fn clear_depth_f32(&self, depth: f32) {
        let gl = &self.raw;
        gl.ClearDepthf(depth);
    }

    unsafe fn clear_stencil(&self, stencil: i32) {
        let gl = &self.raw;
        gl.ClearStencil(stencil);
    }

    unsafe fn clear(&self, mask: u32) {
        let gl = &self.raw;
        gl.Clear(mask);
    }

    unsafe fn pixel_store_i32(&self, parameter: u32, value: i32) {
        let gl = &self.raw;
        gl.PixelStorei(parameter, value);
    }

    unsafe fn pixel_store_bool(&self, parameter: u32, value: bool) {
        let gl = &self.raw;
        gl.PixelStorei(parameter, value as i32);
    }

    unsafe fn buffer_data_size(&self, target: u32, size: i32, usage: u32) {
        let gl = &self.raw;
        gl.BufferData(target, size as isize, std::ptr::null(), usage);
    }

    unsafe fn buffer_data_u8_slice(&self, target: u32, data: &[u8], usage: u32) {
        let gl = &self.raw;
        gl.BufferData(
            target,
            data.len() as isize,
            data.as_ptr() as *const std::ffi::c_void,
            usage,
        );
    }

    unsafe fn named_buffer_data_u8_slice(&self, buffer: Self::Buffer, data: &[u8], usage: u32) {
        let gl = &self.raw;
        gl.NamedBufferData(
            buffer.0.get(),
            data.len() as isize,
            data.as_ptr() as *const std::ffi::c_void,
            usage,
        );
    }

    unsafe fn buffer_sub_data_u8_slice(&self, target: u32, offset: i32, src_data: &[u8]) {
        let gl = &self.raw;
        gl.BufferSubData(
            target,
            offset as isize,
            src_data.len() as isize,
            src_data.as_ptr() as *const std::ffi::c_void,
        );
    }

    unsafe fn get_buffer_sub_data(&self, target: u32, offset: i32, dst_data: &mut [u8]) {
        let gl = &self.raw;
        gl.GetBufferSubData(
            target,
            offset as isize,
            dst_data.len() as isize,
            dst_data.as_mut_ptr() as *mut std::ffi::c_void,
        );
    }

    unsafe fn check_framebuffer_status(&self, target: u32) -> u32 {
        let gl = &self.raw;
        gl.CheckFramebufferStatus(target)
    }

    unsafe fn clear_buffer_i32_slice(&self, target: u32, draw_buffer: u32, values: &[i32]) {
        let gl = &self.raw;
        gl.ClearBufferiv(target, draw_buffer as i32, values.as_ptr());
    }

    unsafe fn clear_buffer_u32_slice(&self, target: u32, draw_buffer: u32, values: &[u32]) {
        let gl = &self.raw;
        gl.ClearBufferuiv(target, draw_buffer as i32, values.as_ptr());
    }

    unsafe fn clear_buffer_f32_slice(&self, target: u32, draw_buffer: u32, values: &[f32]) {
        let gl = &self.raw;
        gl.ClearBufferfv(target, draw_buffer as i32, values.as_ptr());
    }

    unsafe fn clear_buffer_depth_stencil(
        &self,
        target: u32,
        draw_buffer: u32,
        depth: f32,
        stencil: i32,
    ) {
        let gl = &self.raw;
        gl.ClearBufferfi(target, draw_buffer as i32, depth, stencil);
    }

    unsafe fn client_wait_sync(&self, fence: Self::Fence, flags: u32, timeout: i32) -> u32 {
        let gl = &self.raw;
        gl.ClientWaitSync(fence.0, flags, timeout as u64)
    }

    unsafe fn wait_sync(&self, fence: Self::Fence, flags: u32, timeout: u64) {
        let gl = &self.raw;
        gl.WaitSync(fence.0, flags, timeout)
    }

    unsafe fn copy_buffer_sub_data(
        &self,
        src_target: u32,
        dst_target: u32,
        src_offset: i32,
        dst_offset: i32,
        size: i32,
    ) {
        let gl = &self.raw;
        gl.CopyBufferSubData(
            src_target,
            dst_target,
            src_offset as isize,
            dst_offset as isize,
            size as isize,
        );
    }

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
    ) {
        let gl = &self.raw;
        gl.CopyImageSubData(
            src_name.0.get(),
            src_target,
            src_level,
            src_x,
            src_y,
            src_z,
            dst_name.0.get(),
            dst_target,
            dst_level,
            dst_x,
            dst_y,
            dst_z,
            src_width,
            src_height,
            src_depth,
        );
    }

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
    ) {
        let gl = &self.raw;
        gl.CopyTexImage2D(target, level, internal_format, x, y, width, height, border);
    }

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
    ) {
        let gl = &self.raw;
        gl.CopyTexSubImage2D(target, level, x_offset, y_offset, x, y, width, height);
    }

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
    ) {
        let gl = &self.raw;
        gl.CopyTexSubImage3D(
            target, level, x_offset, y_offset, z_offset, x, y, width, height,
        );
    }

    unsafe fn delete_buffer(&self, buffer: Self::Buffer) {
        let gl = &self.raw;
        gl.DeleteBuffers(1, &buffer.0.get());
    }

    unsafe fn delete_framebuffer(&self, framebuffer: Self::Framebuffer) {
        let gl = &self.raw;
        gl.DeleteFramebuffers(1, &framebuffer.0.get());
    }

    unsafe fn delete_query(&self, query: Self::Query) {
        let gl = &self.raw;
        gl.DeleteQueries(1, &query.0.get());
    }

    unsafe fn delete_renderbuffer(&self, renderbuffer: Self::Renderbuffer) {
        let gl = &self.raw;
        gl.DeleteRenderbuffers(1, &renderbuffer.0.get());
    }

    unsafe fn delete_sampler(&self, sampler: Self::Sampler) {
        let gl = &self.raw;
        gl.DeleteSamplers(1, &sampler.0.get());
    }

    unsafe fn delete_sync(&self, fence: Self::Fence) {
        let gl = &self.raw;
        gl.DeleteSync(fence.0);
    }

    unsafe fn delete_texture(&self, texture: Self::Texture) {
        let gl = &self.raw;
        gl.DeleteTextures(1, &texture.0.get());
    }

    unsafe fn disable(&self, parameter: u32) {
        let gl = &self.raw;
        gl.Disable(parameter);
    }

    unsafe fn disable_vertex_attrib_array(&self, index: u32) {
        let gl = &self.raw;
        gl.DisableVertexAttribArray(index);
    }

    unsafe fn draw_arrays(&self, mode: u32, first: i32, count: i32) {
        let gl = &self.raw;
        gl.DrawArrays(mode as u32, first, count);
    }

    unsafe fn draw_arrays_instanced(&self, mode: u32, first: i32, count: i32, instance_count: i32) {
        let gl = &self.raw;
        gl.DrawArraysInstanced(mode as u32, first, count, instance_count);
    }

    unsafe fn draw_buffer(&self, draw_buffer: u32) {
        let gl = &self.raw;
        gl.DrawBuffer(draw_buffer);
    }

    unsafe fn draw_buffers(&self, buffers: &[u32]) {
        let gl = &self.raw;
        gl.DrawBuffers(buffers.len() as i32, buffers.as_ptr());
    }

    unsafe fn draw_elements(&self, mode: u32, count: i32, element_type: u32, offset: i32) {
        let gl = &self.raw;
        gl.DrawElements(
            mode as u32,
            count,
            element_type as u32,
            offset as *const std::ffi::c_void,
        );
    }

    unsafe fn draw_elements_instanced(
        &self,
        mode: u32,
        count: i32,
        element_type: u32,
        offset: i32,
        instance_count: i32,
    ) {
        let gl = &self.raw;
        gl.DrawElementsInstanced(
            mode as u32,
            count,
            element_type as u32,
            offset as *const std::ffi::c_void,
            instance_count,
        );
    }

    unsafe fn enable(&self, parameter: u32) {
        let gl = &self.raw;
        gl.Enable(parameter);
    }

    unsafe fn is_enabled(&self, parameter: u32) -> bool {
        let gl = &self.raw;
        gl.IsEnabled(parameter) != 0
    }

    unsafe fn enable_vertex_array_attrib(&self, vao: Self::VertexArray, index: u32) {
        let gl = &self.raw;
        gl.EnableVertexArrayAttrib(vao.0.get(), index);
    }

    unsafe fn enable_vertex_attrib_array(&self, index: u32) {
        let gl = &self.raw;
        gl.EnableVertexAttribArray(index);
    }

    unsafe fn flush(&self) {
        let gl = &self.raw;
        gl.Flush();
    }

    unsafe fn framebuffer_renderbuffer(
        &self,
        target: u32,
        attachment: u32,
        renderbuffer_target: u32,
        renderbuffer: Option<Self::Renderbuffer>,
    ) {
        let gl = &self.raw;
        gl.FramebufferRenderbuffer(
            target,
            attachment,
            renderbuffer_target,
            renderbuffer.map(|rb| rb.0.get()).unwrap_or(0),
        );
    }

    unsafe fn framebuffer_texture_2d(
        &self,
        target: u32,
        attachment: u32,
        texture_target: u32,
        texture: Option<Self::Texture>,
        level: i32,
    ) {
        let gl = &self.raw;
        gl.FramebufferTexture2D(
            target,
            attachment,
            texture_target,
            texture.map(|t| t.0.get()).unwrap_or(0),
            level,
        );
    }

    unsafe fn framebuffer_texture_layer(
        &self,
        target: u32,
        attachment: u32,
        texture: Option<Self::Texture>,
        level: i32,
        layer: i32,
    ) {
        let gl = &self.raw;
        gl.FramebufferTextureLayer(
            target,
            attachment,
            texture.map(|t| t.0.get()).unwrap_or(0),
            level,
            layer,
        );
    }

    unsafe fn front_face(&self, value: u32) {
        let gl = &self.raw;
        gl.FrontFace(value as u32);
    }

    unsafe fn get_error(&self) -> u32 {
        let gl = &self.raw;
        gl.GetError()
    }

    unsafe fn get_tex_parameter_i32(&self, target: u32, parameter: u32) -> i32 {
        let gl = &self.raw;
        let mut value = 0;
        gl.GetTexParameteriv(target, parameter, &mut value);
        value
    }

    unsafe fn get_buffer_parameter_i32(&self, target: u32, parameter: u32) -> i32 {
        let gl = &self.raw;
        let mut value = 0;
        gl.GetBufferParameteriv(target, parameter, &mut value);
        value
    }

    unsafe fn get_parameter_i32(&self, parameter: u32) -> i32 {
        let gl = &self.raw;
        let mut value = 0;
        gl.GetIntegerv(parameter, &mut value);
        value
    }

    unsafe fn get_parameter_i32_slice(&self, parameter: u32, out: &mut [i32]) {
        let gl = &self.raw;
        gl.GetIntegerv(parameter, &mut out[0]);
    }

    unsafe fn get_parameter_f32(&self, parameter: u32) -> f32 {
        let gl = &self.raw;
        let mut value: f32 = 0.0;
        gl.GetFloatv(parameter, &mut value);
        value
    }

    unsafe fn get_parameter_f32_slice(&self, parameter: u32, out: &mut [f32]) {
        let gl = &self.raw;
        gl.GetFloatv(parameter, &mut out[0]);
    }

    unsafe fn get_parameter_indexed_i32(&self, parameter: u32, index: u32) -> i32 {
        let gl = &self.raw;
        let mut value = 0;
        gl.GetIntegeri_v(parameter, index, &mut value);
        value
    }

    unsafe fn get_parameter_indexed_string(&self, parameter: u32, index: u32) -> String {
        let gl = &self.raw;
        let raw_ptr = gl.GetStringi(parameter, index);
        std::ffi::CStr::from_ptr(raw_ptr as *const native_gl::GLchar)
            .to_str()
            .unwrap()
            .to_owned()
    }

    unsafe fn get_parameter_string(&self, parameter: u32) -> String {
        let gl = &self.raw;
        let raw_ptr = gl.GetString(parameter);
        if raw_ptr.is_null() {
            panic!(
                "Get parameter string 0x{:X} failed. Maybe your GL context version is too outdated.",
                parameter
            )
        }
        std::ffi::CStr::from_ptr(raw_ptr as *const native_gl::GLchar)
            .to_str()
            .unwrap()
            .to_owned()
    }

    unsafe fn get_uniform_location(
        &self,
        program: Self::Program,
        name: &str,
    ) -> Option<Self::UniformLocation> {
        let gl = &self.raw;
        let name = CString::new(name).unwrap();
        let uniform_location =
            gl.GetUniformLocation(program.0.get(), name.as_ptr() as *const native_gl::GLchar);
        if uniform_location < 0 {
            None
        } else {
            Some(NativeUniformLocation(uniform_location as u32))
        }
    }

    unsafe fn get_attrib_location(&self, program: Self::Program, name: &str) -> Option<u32> {
        let gl = &self.raw;
        let name = CString::new(name).unwrap();
        let attrib_location =
            gl.GetAttribLocation(program.0.get(), name.as_ptr() as *const native_gl::GLchar);
        if attrib_location < 0 {
            None
        } else {
            Some(attrib_location as u32)
        }
    }

    unsafe fn bind_attrib_location(&self, program: Self::Program, index: u32, name: &str) {
        let gl = &self.raw;
        let name = CString::new(name).unwrap();
        gl.BindAttribLocation(
            program.0.get(),
            index,
            name.as_ptr() as *const native_gl::GLchar,
        );
    }

    unsafe fn get_active_attributes(&self, program: Self::Program) -> u32 {
        let gl = &self.raw;
        let mut count = 0;
        gl.GetProgramiv(program.0.get(), ACTIVE_ATTRIBUTES, &mut count);
        count as u32
    }

    unsafe fn get_active_attribute(
        &self,
        program: Self::Program,
        index: u32,
    ) -> Option<ActiveAttribute> {
        let gl = &self.raw;
        let mut attribute_max_size = 0;
        gl.GetProgramiv(
            program.0.get(),
            ACTIVE_ATTRIBUTE_MAX_LENGTH,
            &mut attribute_max_size,
        );
        let mut name = String::with_capacity(attribute_max_size as usize);
        name.extend(std::iter::repeat('\0').take(attribute_max_size as usize));
        let mut length = 0;
        let mut size = 0;
        let mut atype = 0;
        gl.GetActiveAttrib(
            program.0.get(),
            index,
            attribute_max_size,
            &mut length,
            &mut size,
            &mut atype,
            name.as_ptr() as *mut native_gl::GLchar,
        );

        name.truncate(length as usize);

        Some(ActiveAttribute { name, size, atype })
    }

    unsafe fn get_sync_status(&self, fence: Self::Fence) -> u32 {
        let gl = &self.raw;
        let mut len = 0;
        let mut values = [UNSIGNALED as i32];
        gl.GetSynciv(
            fence.0,
            SYNC_STATUS,
            values.len() as i32,
            &mut len,
            values.as_mut_ptr(),
        );
        values[0] as u32
    }

    unsafe fn is_sync(&self, fence: Self::Fence) -> bool {
        let gl = &self.raw;
        1 == gl.IsSync(fence.0)
    }

    unsafe fn renderbuffer_storage(
        &self,
        target: u32,
        internal_format: u32,
        width: i32,
        height: i32,
    ) {
        let gl = &self.raw;
        gl.RenderbufferStorage(target, internal_format, width, height);
    }

    unsafe fn renderbuffer_storage_multisample(
        &self,
        target: u32,
        samples: i32,
        internal_format: u32,
        width: i32,
        height: i32,
    ) {
        let gl = &self.raw;
        gl.RenderbufferStorageMultisample(target, samples, internal_format, width, height);
    }

    unsafe fn sampler_parameter_f32(&self, sampler: Self::Sampler, name: u32, value: f32) {
        let gl = &self.raw;
        gl.SamplerParameterf(sampler.0.get(), name, value);
    }

    unsafe fn sampler_parameter_i32(&self, sampler: Self::Sampler, name: u32, value: i32) {
        let gl = &self.raw;
        gl.SamplerParameteri(sampler.0.get(), name, value);
    }

    unsafe fn generate_mipmap(&self, target: u32) {
        let gl = &self.raw;
        gl.GenerateMipmap(target);
    }

    unsafe fn generate_texture_mipmap(&self, texture: Self::Texture) {
        let gl = &self.raw;
        gl.GenerateTextureMipmap(texture.0.get());
    }

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
    ) {
        let gl = &self.raw;
        gl.TexImage2D(
            target,
            level,
            internal_format,
            width,
            height,
            border,
            format,
            ty,
            pixels.map(|p| p.as_ptr()).unwrap_or(std::ptr::null()) as *const std::ffi::c_void,
        );
    }

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
    ) {
        let gl = &self.raw;
        gl.CompressedTexImage2D(
            target,
            level,
            internal_format as u32,
            width,
            height,
            border,
            image_size,
            pixels.as_ptr() as *const std::ffi::c_void,
        );
    }

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
    ) {
        let gl = &self.raw;
        gl.TexImage3D(
            target,
            level,
            internal_format,
            width,
            height,
            depth,
            border,
            format,
            ty,
            pixels.map(|p| p.as_ptr()).unwrap_or(std::ptr::null()) as *const std::ffi::c_void,
        );
    }

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
    ) {
        let gl = &self.raw;
        gl.CompressedTexImage3D(
            target,
            level,
            internal_format as u32,
            width,
            height,
            depth,
            border,
            image_size,
            pixels.as_ptr() as *const std::ffi::c_void,
        );
    }

    unsafe fn tex_storage_2d(
        &self,
        target: u32,
        levels: i32,
        internal_format: u32,
        width: i32,
        height: i32,
    ) {
        let gl = &self.raw;
        gl.TexStorage2D(target, levels, internal_format, width, height);
    }

    unsafe fn tex_storage_3d(
        &self,
        target: u32,
        levels: i32,
        internal_format: u32,
        width: i32,
        height: i32,
        depth: i32,
    ) {
        let gl = &self.raw;
        gl.TexStorage3D(target, levels, internal_format, width, height, depth);
    }

    unsafe fn texture_storage_3d(
        &self,
        texture: Self::Texture,
        levels: i32,
        internal_format: u32,
        width: i32,
        height: i32,
        depth: i32,
    ) {
        let gl = &self.raw;
        gl.TextureStorage3D(
            texture.0.get(),
            levels,
            internal_format,
            width,
            height,
            depth,
        );
    }

    unsafe fn get_uniform_i32(
        &self,
        program: Self::Program,
        location: &Self::UniformLocation,
        v: &mut [i32],
    ) {
        let gl = &self.raw;
        gl.GetUniformiv(
            program.0.get() as u32,
            location.0 as i32,
            v.as_mut_ptr() as *mut i32,
        )
    }

    unsafe fn get_uniform_f32(
        &self,
        program: Self::Program,
        location: &Self::UniformLocation,
        v: &mut [f32],
    ) {
        let gl = &self.raw;
        gl.GetUniformfv(
            program.0.get() as u32,
            location.0 as i32,
            v.as_mut_ptr() as *mut f32,
        )
    }

    unsafe fn uniform_1_i32(&self, location: Option<&Self::UniformLocation>, x: i32) {
        let gl = &self.raw;
        if let Some(loc) = location {
            gl.Uniform1i(loc.0 as i32, x);
        }
    }

    unsafe fn uniform_2_i32(&self, location: Option<&Self::UniformLocation>, x: i32, y: i32) {
        let gl = &self.raw;
        if let Some(loc) = location {
            gl.Uniform2i(loc.0 as i32, x, y);
        }
    }

    unsafe fn uniform_3_i32(
        &self,
        location: Option<&Self::UniformLocation>,
        x: i32,
        y: i32,
        z: i32,
    ) {
        let gl = &self.raw;
        if let Some(loc) = location {
            gl.Uniform3i(loc.0 as i32, x, y, z);
        }
    }

    unsafe fn uniform_4_i32(
        &self,
        location: Option<&Self::UniformLocation>,
        x: i32,
        y: i32,
        z: i32,
        w: i32,
    ) {
        let gl = &self.raw;
        if let Some(loc) = location {
            gl.Uniform4i(loc.0 as i32, x, y, z, w);
        }
    }

    unsafe fn uniform_1_i32_slice(&self, location: Option<&Self::UniformLocation>, v: &[i32]) {
        let gl = &self.raw;
        if let Some(loc) = location {
            gl.Uniform1iv(loc.0 as i32, v.len() as i32, v.as_ptr());
        }
    }

    unsafe fn uniform_2_i32_slice(&self, location: Option<&Self::UniformLocation>, v: &[i32]) {
        let gl = &self.raw;
        if let Some(loc) = location {
            gl.Uniform2iv(loc.0 as i32, v.len() as i32 / 2, v.as_ptr());
        }
    }

    unsafe fn uniform_3_i32_slice(&self, location: Option<&Self::UniformLocation>, v: &[i32]) {
        let gl = &self.raw;
        if let Some(loc) = location {
            gl.Uniform3iv(loc.0 as i32, v.len() as i32 / 3, v.as_ptr());
        }
    }

    unsafe fn uniform_4_i32_slice(&self, location: Option<&Self::UniformLocation>, v: &[i32]) {
        let gl = &self.raw;
        if let Some(loc) = location {
            gl.Uniform4iv(loc.0 as i32, v.len() as i32 / 4, v.as_ptr());
        }
    }

    unsafe fn uniform_1_u32(&self, location: Option<&Self::UniformLocation>, x: u32) {
        let gl = &self.raw;
        if let Some(loc) = location {
            gl.Uniform1ui(loc.0 as i32, x);
        }
    }

    unsafe fn uniform_2_u32(&self, location: Option<&Self::UniformLocation>, x: u32, y: u32) {
        let gl = &self.raw;
        if let Some(loc) = location {
            gl.Uniform2ui(loc.0 as i32, x, y);
        }
    }

    unsafe fn uniform_3_u32(
        &self,
        location: Option<&Self::UniformLocation>,
        x: u32,
        y: u32,
        z: u32,
    ) {
        let gl = &self.raw;
        if let Some(loc) = location {
            gl.Uniform3ui(loc.0 as i32, x, y, z);
        }
    }

    unsafe fn uniform_4_u32(
        &self,
        location: Option<&Self::UniformLocation>,
        x: u32,
        y: u32,
        z: u32,
        w: u32,
    ) {
        let gl = &self.raw;
        if let Some(loc) = location {
            gl.Uniform4ui(loc.0 as i32, x, y, z, w);
        }
    }

    unsafe fn uniform_1_u32_slice(&self, location: Option<&Self::UniformLocation>, v: &[u32]) {
        let gl = &self.raw;
        if let Some(loc) = location {
            gl.Uniform1uiv(loc.0 as i32, v.len() as i32, v.as_ptr());
        }
    }

    unsafe fn uniform_2_u32_slice(&self, location: Option<&Self::UniformLocation>, v: &[u32]) {
        let gl = &self.raw;
        if let Some(loc) = location {
            gl.Uniform2uiv(loc.0 as i32, v.len() as i32 / 2, v.as_ptr());
        }
    }

    unsafe fn uniform_3_u32_slice(&self, location: Option<&Self::UniformLocation>, v: &[u32]) {
        let gl = &self.raw;
        if let Some(loc) = location {
            gl.Uniform3uiv(loc.0 as i32, v.len() as i32 / 3, v.as_ptr());
        }
    }

    unsafe fn uniform_4_u32_slice(&self, location: Option<&Self::UniformLocation>, v: &[u32]) {
        let gl = &self.raw;
        if let Some(loc) = location {
            gl.Uniform4uiv(loc.0 as i32, v.len() as i32 / 4, v.as_ptr());
        }
    }

    unsafe fn uniform_1_f32(&self, location: Option<&Self::UniformLocation>, x: f32) {
        let gl = &self.raw;
        if let Some(loc) = location {
            gl.Uniform1f(loc.0 as i32, x);
        }
    }

    unsafe fn uniform_2_f32(&self, location: Option<&Self::UniformLocation>, x: f32, y: f32) {
        let gl = &self.raw;
        if let Some(loc) = location {
            gl.Uniform2f(loc.0 as i32, x, y);
        }
    }

    unsafe fn uniform_3_f32(
        &self,
        location: Option<&Self::UniformLocation>,
        x: f32,
        y: f32,
        z: f32,
    ) {
        let gl = &self.raw;
        if let Some(loc) = location {
            gl.Uniform3f(loc.0 as i32, x, y, z);
        }
    }

    unsafe fn uniform_4_f32(
        &self,
        location: Option<&Self::UniformLocation>,
        x: f32,
        y: f32,
        z: f32,
        w: f32,
    ) {
        let gl = &self.raw;
        if let Some(loc) = location {
            gl.Uniform4f(loc.0 as i32, x, y, z, w);
        }
    }

    unsafe fn uniform_1_f32_slice(&self, location: Option<&Self::UniformLocation>, v: &[f32]) {
        let gl = &self.raw;
        if let Some(loc) = location {
            gl.Uniform1fv(loc.0 as i32, v.len() as i32, v.as_ptr());
        }
    }

    unsafe fn uniform_2_f32_slice(&self, location: Option<&Self::UniformLocation>, v: &[f32]) {
        let gl = &self.raw;
        if let Some(loc) = location {
            gl.Uniform2fv(loc.0 as i32, v.len() as i32 / 2, v.as_ptr());
        }
    }

    unsafe fn uniform_3_f32_slice(&self, location: Option<&Self::UniformLocation>, v: &[f32]) {
        let gl = &self.raw;
        if let Some(loc) = location {
            gl.Uniform3fv(loc.0 as i32, v.len() as i32 / 3, v.as_ptr());
        }
    }

    unsafe fn uniform_4_f32_slice(&self, location: Option<&Self::UniformLocation>, v: &[f32]) {
        let gl = &self.raw;
        if let Some(loc) = location {
            gl.Uniform4fv(loc.0 as i32, v.len() as i32 / 4, v.as_ptr());
        }
    }

    unsafe fn uniform_matrix_2_f32_slice(
        &self,
        location: Option<&Self::UniformLocation>,
        transpose: bool,
        v: &[f32],
    ) {
        let gl = &self.raw;
        if let Some(loc) = location {
            gl.UniformMatrix2fv(
                loc.0 as i32,
                v.len() as i32 / 4,
                transpose as u8,
                v.as_ptr(),
            );
        }
    }

    unsafe fn uniform_matrix_2x3_f32_slice(
        &self,
        location: Option<&Self::UniformLocation>,
        transpose: bool,
        v: &[f32],
    ) {
        let gl = &self.raw;
        if let Some(loc) = location {
            gl.UniformMatrix2x3fv(
                loc.0 as i32,
                v.len() as i32 / 6,
                transpose as u8,
                v.as_ptr(),
            );
        }
    }

    unsafe fn uniform_matrix_2x4_f32_slice(
        &self,
        location: Option<&Self::UniformLocation>,
        transpose: bool,
        v: &[f32],
    ) {
        let gl = &self.raw;
        if let Some(loc) = location {
            gl.UniformMatrix2x4fv(
                loc.0 as i32,
                v.len() as i32 / 8,
                transpose as u8,
                v.as_ptr(),
            );
        }
    }

    unsafe fn uniform_matrix_3x2_f32_slice(
        &self,
        location: Option<&Self::UniformLocation>,
        transpose: bool,
        v: &[f32],
    ) {
        let gl = &self.raw;
        if let Some(loc) = location {
            gl.UniformMatrix3x2fv(
                loc.0 as i32,
                v.len() as i32 / 6,
                transpose as u8,
                v.as_ptr(),
            );
        }
    }

    unsafe fn uniform_matrix_3_f32_slice(
        &self,
        location: Option<&Self::UniformLocation>,
        transpose: bool,
        v: &[f32],
    ) {
        let gl = &self.raw;
        if let Some(loc) = location {
            gl.UniformMatrix3fv(
                loc.0 as i32,
                v.len() as i32 / 9,
                transpose as u8,
                v.as_ptr(),
            );
        }
    }

    unsafe fn uniform_matrix_3x4_f32_slice(
        &self,
        location: Option<&Self::UniformLocation>,
        transpose: bool,
        v: &[f32],
    ) {
        let gl = &self.raw;
        if let Some(loc) = location {
            gl.UniformMatrix3x4fv(
                loc.0 as i32,
                v.len() as i32 / 12,
                transpose as u8,
                v.as_ptr(),
            );
        }
    }

    unsafe fn uniform_matrix_4x2_f32_slice(
        &self,
        location: Option<&Self::UniformLocation>,
        transpose: bool,
        v: &[f32],
    ) {
        let gl = &self.raw;
        if let Some(loc) = location {
            gl.UniformMatrix4x2fv(
                loc.0 as i32,
                v.len() as i32 / 8,
                transpose as u8,
                v.as_ptr(),
            );
        }
    }

    unsafe fn uniform_matrix_4x3_f32_slice(
        &self,
        location: Option<&Self::UniformLocation>,
        transpose: bool,
        v: &[f32],
    ) {
        let gl = &self.raw;
        if let Some(loc) = location {
            gl.UniformMatrix4x3fv(
                loc.0 as i32,
                v.len() as i32 / 12,
                transpose as u8,
                v.as_ptr(),
            );
        }
    }

    unsafe fn uniform_matrix_4_f32_slice(
        &self,
        location: Option<&Self::UniformLocation>,
        transpose: bool,
        v: &[f32],
    ) {
        let gl = &self.raw;
        if let Some(loc) = location {
            gl.UniformMatrix4fv(
                loc.0 as i32,
                v.len() as i32 / 16,
                transpose as u8,
                v.as_ptr(),
            );
        }
    }

    unsafe fn cull_face(&self, value: u32) {
        let gl = &self.raw;
        gl.CullFace(value as u32);
    }

    unsafe fn color_mask(&self, red: bool, green: bool, blue: bool, alpha: bool) {
        let gl = &self.raw;
        gl.ColorMask(red as u8, green as u8, blue as u8, alpha as u8);
    }

    unsafe fn depth_mask(&self, value: bool) {
        let gl = &self.raw;
        gl.DepthMask(value as u8);
    }

    unsafe fn blend_color(&self, red: f32, green: f32, blue: f32, alpha: f32) {
        let gl = &self.raw;
        gl.BlendColor(red, green, blue, alpha);
    }

    unsafe fn line_width(&self, width: f32) {
        let gl = &self.raw;
        gl.LineWidth(width);
    }

    unsafe fn invalidate_framebuffer(&self, target: u32, attachments: &[u32]) {
        let gl = &self.raw;
        gl.InvalidateFramebuffer(target, attachments.len() as i32, attachments.as_ptr());
    }

    unsafe fn polygon_offset(&self, factor: f32, units: f32) {
        let gl = &self.raw;
        gl.PolygonOffset(factor, units);
    }

    unsafe fn polygon_mode(&self, face: u32, mode: u32) {
        let gl = &self.raw;
        gl.PolygonMode(face as u32, mode as u32);
    }

    unsafe fn finish(&self) {
        let gl = &self.raw;
        gl.Finish();
    }

    unsafe fn bind_texture(&self, target: u32, texture: Option<Self::Texture>) {
        let gl = &self.raw;
        gl.BindTexture(target, texture.map(|t| t.0.get()).unwrap_or(0));
    }

    unsafe fn bind_sampler(&self, unit: u32, sampler: Option<Self::Sampler>) {
        let gl = &self.raw;
        gl.BindSampler(unit, sampler.map(|s| s.0.get()).unwrap_or(0));
    }

    unsafe fn active_texture(&self, unit: u32) {
        let gl = &self.raw;
        gl.ActiveTexture(unit);
    }

    unsafe fn fence_sync(&self, condition: u32, flags: u32) -> Result<Self::Fence, String> {
        let gl = &self.raw;
        Ok(NativeFence(gl.FenceSync(condition as u32, flags)))
    }

    unsafe fn tex_parameter_f32(&self, target: u32, parameter: u32, value: f32) {
        let gl = &self.raw;
        gl.TexParameterf(target, parameter, value);
    }

    unsafe fn tex_parameter_i32(&self, target: u32, parameter: u32, value: i32) {
        let gl = &self.raw;
        gl.TexParameteri(target, parameter, value);
    }

    unsafe fn texture_parameter_i32(&self, texture: Self::Texture, parameter: u32, value: i32) {
        let gl = &self.raw;
        gl.TextureParameteri(texture.0.get(), parameter, value);
    }

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
    ) {
        let gl = &self.raw;
        gl.TexSubImage2D(
            target,
            level,
            x_offset,
            y_offset,
            width,
            height,
            format,
            ty,
            match pixels {
                PixelUnpackData::BufferOffset(offset) => offset as *const std::ffi::c_void,
                PixelUnpackData::Slice(data) => data.as_ptr() as *const std::ffi::c_void,
            },
        );
    }

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
    ) {
        let gl = &self.raw;
        let (data, image_size) = match pixels {
            CompressedPixelUnpackData::BufferRange(ref range) => (
                range.start as *const std::ffi::c_void,
                (range.end - range.start) as i32,
            ),
            CompressedPixelUnpackData::Slice(data) => {
                (data.as_ptr() as *const std::ffi::c_void, data.len() as i32)
            }
        };

        gl.CompressedTexSubImage2D(
            target, level, x_offset, y_offset, width, height, format, image_size, data,
        );
    }

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
    ) {
        let gl = &self.raw;
        gl.TexSubImage3D(
            target,
            level,
            x_offset,
            y_offset,
            z_offset,
            width,
            height,
            depth,
            format,
            ty,
            match pixels {
                PixelUnpackData::BufferOffset(offset) => offset as *const std::ffi::c_void,
                PixelUnpackData::Slice(data) => data.as_ptr() as *const std::ffi::c_void,
            },
        );
    }

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
    ) {
        let gl = &self.raw;
        gl.TextureSubImage3D(
            texture.0.get(),
            level,
            x_offset,
            y_offset,
            z_offset,
            width,
            height,
            depth,
            format,
            ty,
            match pixels {
                PixelUnpackData::BufferOffset(offset) => offset as *const std::ffi::c_void,
                PixelUnpackData::Slice(data) => data.as_ptr() as *const std::ffi::c_void,
            },
        );
    }

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
    ) {
        let gl = &self.raw;
        let (data, image_size) = match pixels {
            CompressedPixelUnpackData::BufferRange(ref range) => (
                range.start as *const std::ffi::c_void,
                (range.end - range.start) as i32,
            ),
            CompressedPixelUnpackData::Slice(data) => {
                (data.as_ptr() as *const std::ffi::c_void, data.len() as i32)
            }
        };

        gl.CompressedTexSubImage3D(
            target, level, x_offset, y_offset, z_offset, width, height, depth, format, image_size,
            data,
        );
    }

    unsafe fn depth_func(&self, func: u32) {
        let gl = &self.raw;
        gl.DepthFunc(func as u32);
    }

    unsafe fn depth_range_f32(&self, near: f32, far: f32) {
        let gl = &self.raw;
        gl.DepthRangef(near, far);
    }

    unsafe fn scissor(&self, x: i32, y: i32, width: i32, height: i32) {
        let gl = &self.raw;
        gl.Scissor(x, y, width, height);
    }

    unsafe fn vertex_array_attrib_binding_f32(
        &self,
        vao: Self::VertexArray,
        index: u32,
        binding_index: u32,
    ) {
        let gl = &self.raw;
        gl.VertexArrayAttribBinding(vao.0.get(), index, binding_index);
    }

    unsafe fn vertex_array_attrib_format_f32(
        &self,
        vao: Self::VertexArray,
        index: u32,
        size: i32,
        data_type: u32,
        normalized: bool,
        relative_offset: u32,
    ) {
        let gl = &self.raw;
        gl.VertexArrayAttribFormat(
            vao.0.get(),
            index,
            size,
            data_type,
            normalized as u8,
            relative_offset,
        );
    }

    unsafe fn vertex_array_attrib_format_i32(
        &self,
        vao: Self::VertexArray,
        index: u32,
        size: i32,
        data_type: u32,
        relative_offset: u32,
    ) {
        let gl = &self.raw;
        gl.VertexArrayAttribIFormat(vao.0.get(), index, size, data_type, relative_offset);
    }

    unsafe fn vertex_array_element_buffer(
        &self,
        vao: Self::VertexArray,
        buffer: Option<Self::Buffer>,
    ) {
        let gl = &self.raw;
        gl.VertexArrayElementBuffer(vao.0.get(), buffer.map(|b| b.0.get()).unwrap_or(0));
    }

    unsafe fn vertex_array_vertex_buffer(
        &self,
        vao: Self::VertexArray,
        binding_index: u32,
        buffer: Option<Self::Buffer>,
        offset: i32,
        stride: i32,
    ) {
        let gl = &self.raw;
        gl.VertexArrayVertexBuffer(
            vao.0.get(),
            binding_index,
            buffer.map(|b| b.0.get()).unwrap_or(0),
            offset as isize,
            stride,
        );
    }

    unsafe fn vertex_attrib_divisor(&self, index: u32, divisor: u32) {
        let gl = &self.raw;
        gl.VertexAttribDivisor(index, divisor);
    }

    unsafe fn vertex_attrib_pointer_f32(
        &self,
        index: u32,
        size: i32,
        data_type: u32,
        normalized: bool,
        stride: i32,
        offset: i32,
    ) {
        let gl = &self.raw;
        gl.VertexAttribPointer(
            index,
            size,
            data_type,
            normalized as u8,
            stride,
            offset as *const std::ffi::c_void,
        );
    }

    unsafe fn vertex_attrib_pointer_i32(
        &self,
        index: u32,
        size: i32,
        data_type: u32,
        stride: i32,
        offset: i32,
    ) {
        let gl = &self.raw;
        gl.VertexAttribIPointer(
            index,
            size,
            data_type,
            stride,
            offset as *const std::ffi::c_void,
        );
    }

    unsafe fn vertex_attrib_1_f32(&self, index: u32, x: f32) {
        let gl = &self.raw;
        gl.VertexAttrib1f(index, x);
    }

    unsafe fn vertex_attrib_2_f32(&self, index: u32, x: f32, y: f32) {
        let gl = &self.raw;
        gl.VertexAttrib2f(index, x, y);
    }

    unsafe fn vertex_attrib_3_f32(&self, index: u32, x: f32, y: f32, z: f32) {
        let gl = &self.raw;
        gl.VertexAttrib3f(index, x, y, z);
    }

    unsafe fn vertex_attrib_4_f32(&self, index: u32, x: f32, y: f32, z: f32, w: f32) {
        let gl = &self.raw;
        gl.VertexAttrib4f(index, x, y, z, w);
    }

    unsafe fn vertex_attrib_1_f32_slice(&self, index: u32, v: &[f32]) {
        let gl = &self.raw;
        gl.VertexAttrib1fv(index, v.as_ptr());
    }

    unsafe fn vertex_attrib_2_f32_slice(&self, index: u32, v: &[f32]) {
        let gl = &self.raw;
        gl.VertexAttrib2fv(index, v.as_ptr());
    }

    unsafe fn vertex_attrib_3_f32_slice(&self, index: u32, v: &[f32]) {
        let gl = &self.raw;
        gl.VertexAttrib3fv(index, v.as_ptr());
    }

    unsafe fn vertex_attrib_4_f32_slice(&self, index: u32, v: &[f32]) {
        let gl = &self.raw;
        gl.VertexAttrib4fv(index, v.as_ptr());
    }

    unsafe fn viewport(&self, x: i32, y: i32, width: i32, height: i32) {
        let gl = &self.raw;
        gl.Viewport(x, y, width, height);
    }

    unsafe fn blend_equation(&self, mode: u32) {
        let gl = &self.raw;
        gl.BlendEquation(mode as u32);
    }

    unsafe fn blend_equation_separate(&self, mode_rgb: u32, mode_alpha: u32) {
        let gl = &self.raw;
        gl.BlendEquationSeparate(mode_rgb as u32, mode_alpha as u32);
    }

    unsafe fn blend_func(&self, src: u32, dst: u32) {
        let gl = &self.raw;
        gl.BlendFunc(src as u32, dst as u32);
    }

    unsafe fn blend_func_separate(
        &self,
        src_rgb: u32,
        dst_rgb: u32,
        src_alpha: u32,
        dst_alpha: u32,
    ) {
        let gl = &self.raw;
        gl.BlendFuncSeparate(
            src_rgb as u32,
            dst_rgb as u32,
            src_alpha as u32,
            dst_alpha as u32,
        );
    }

    unsafe fn stencil_func(&self, func: u32, reference: i32, mask: u32) {
        let gl = &self.raw;
        gl.StencilFunc(func as u32, reference, mask);
    }

    unsafe fn stencil_func_separate(&self, face: u32, func: u32, reference: i32, mask: u32) {
        let gl = &self.raw;
        gl.StencilFuncSeparate(face as u32, func as u32, reference, mask);
    }

    unsafe fn stencil_mask(&self, mask: u32) {
        let gl = &self.raw;
        gl.StencilMask(mask);
    }

    unsafe fn stencil_mask_separate(&self, face: u32, mask: u32) {
        let gl = &self.raw;
        gl.StencilMaskSeparate(face as u32, mask);
    }

    unsafe fn stencil_op(&self, stencil_fail: u32, depth_fail: u32, pass: u32) {
        let gl = &self.raw;
        gl.StencilOp(stencil_fail as u32, depth_fail as u32, pass as u32);
    }

    unsafe fn stencil_op_separate(&self, face: u32, stencil_fail: u32, depth_fail: u32, pass: u32) {
        let gl = &self.raw;
        gl.StencilOpSeparate(
            face as u32,
            stencil_fail as u32,
            depth_fail as u32,
            pass as u32,
        );
    }

    unsafe fn get_uniform_block_index(&self, program: Self::Program, name: &str) -> Option<u32> {
        let gl = &self.raw;
        let name = CString::new(name).unwrap();
        let index = gl.GetUniformBlockIndex(program.0.get(), name.as_ptr());
        if index == INVALID_INDEX {
            None
        } else {
            Some(index)
        }
    }

    unsafe fn uniform_block_binding(&self, program: Self::Program, index: u32, binding: u32) {
        let gl = &self.raw;
        gl.UniformBlockBinding(program.0.get(), index, binding);
    }

    unsafe fn read_buffer(&self, src: u32) {
        let gl = &self.raw;
        gl.ReadBuffer(src);
    }

    unsafe fn read_pixels(
        &self,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        format: u32,
        gltype: u32,
        pixels: PixelPackData,
    ) {
        let gl = &self.raw;
        gl.ReadPixels(
            x,
            y,
            width,
            height,
            format,
            gltype,
            match pixels {
                PixelPackData::BufferOffset(offset) => offset as *mut std::ffi::c_void,
                PixelPackData::Slice(data) => data.as_mut_ptr() as *mut std::ffi::c_void,
            },
        );
    }

    unsafe fn begin_query(&self, target: u32, query: Self::Query) {
        let gl = &self.raw;
        gl.BeginQuery(target, query.0.get());
    }

    unsafe fn end_query(&self, target: u32) {
        let gl = &self.raw;
        gl.EndQuery(target);
    }

    unsafe fn get_query_parameter_u32(&self, query: Self::Query, parameter: u32) -> u32 {
        let gl = &self.raw;
        let mut value = 0;
        gl.GetQueryObjectuiv(query.0.get(), parameter, &mut value);
        value
    }

    unsafe fn create_transform_feedback(&self) -> Result<Self::TransformFeedback, String> {
        let gl = &self.raw;
        let mut name = 0;
        gl.GenTransformFeedbacks(1, &mut name);
        Ok(NativeTransformFeedback(non_zero_gl_name(name)))
    }

    unsafe fn delete_transform_feedback(&self, transform_feedback: Self::TransformFeedback) {
        let gl = &self.raw;
        gl.DeleteTransformFeedbacks(1, &transform_feedback.0.get());
    }

    unsafe fn bind_transform_feedback(
        &self,
        target: u32,
        transform_feedback: Option<Self::TransformFeedback>,
    ) {
        let gl = &self.raw;
        gl.BindTransformFeedback(target, transform_feedback.map(|tf| tf.0.get()).unwrap_or(0));
    }

    unsafe fn begin_transform_feedback(&self, primitive_mode: u32) {
        let gl = &self.raw;
        gl.BeginTransformFeedback(primitive_mode);
    }

    unsafe fn end_transform_feedback(&self) {
        let gl = &self.raw;
        gl.EndTransformFeedback();
    }

    unsafe fn pause_transform_feedback(&self) {
        let gl = &self.raw;
        gl.PauseTransformFeedback();
    }

    unsafe fn resume_transform_feedback(&self) {
        let gl = &self.raw;
        gl.ResumeTransformFeedback();
    }

    unsafe fn transform_feedback_varyings(
        &self,
        program: Self::Program,
        varyings: &[&str],
        buffer_mode: u32,
    ) {
        let gl = &self.raw;

        let strings: Vec<CString> = varyings
            .iter()
            .copied()
            .map(CString::new)
            .collect::<Result<_, _>>()
            .unwrap();
        let varyings: Vec<_> = strings.iter().map(|c_str| c_str.as_ptr()).collect();

        gl.TransformFeedbackVaryings(
            program.0.get(),
            varyings.len() as i32,
            varyings.as_ptr(),
            buffer_mode,
        );
    }

    unsafe fn get_transform_feedback_varying(
        &self,
        program: Self::Program,
        index: u32,
    ) -> Option<ActiveTransformFeedback> {
        let gl = &self.raw;

        const buf_size: usize = 256;
        const bytes: [u8; buf_size] = [0; buf_size];

        let size: i32 = 0;
        let tftype: u32 = 0;
        let c_name = CString::new(bytes.to_vec()).unwrap();
        let c_name_buf = c_name.into_raw();

        gl.GetTransformFeedbackVarying(
            program.0.get(),
            index,
            buf_size as i32,
            std::ptr::null_mut(),
            size as *mut i32,
            tftype as *mut u32,
            c_name_buf,
        );

        let name = CString::from_raw(c_name_buf).into_string().unwrap();

        Some(ActiveTransformFeedback { size, tftype, name })
    }

    unsafe fn get_active_uniform_block_parameter_i32(
        &self,
        program: Self::Program,
        uniform_block_index: u32,
        parameter: u32,
    ) -> i32 {
        let gl = &self.raw;
        let mut value = 0;
        gl.GetActiveUniformBlockiv(program.0.get(), uniform_block_index, parameter, &mut value);
        value
    }

    unsafe fn get_active_uniform_block_parameter_i32_slice(
        &self,
        program: Self::Program,
        uniform_block_index: u32,
        parameter: u32,
        out: &mut [i32],
    ) {
        let gl = &self.raw;
        gl.GetActiveUniformBlockiv(
            program.0.get(),
            uniform_block_index,
            parameter,
            out.as_mut_ptr(),
        );
    }
    unsafe fn get_active_uniform_block_name(
        &self,
        program: Self::Program,
        uniform_block_index: u32,
    ) -> String {
        let gl = &self.raw;

        // Probe for the length of the name of the uniform block, and, failing
        // that, fall back to allocating a buffer that is 256 bytes long. This
        // should be good enough for pretty much all contexts, including faulty
        // or partially faulty ones.
        let len = self.get_active_uniform_block_parameter_i32(
            program,
            uniform_block_index,
            crate::UNIFORM_BLOCK_NAME_LENGTH,
        );
        let len = if gl.GetError() == crate::NO_ERROR && len > 0 {
            len as usize
        } else {
            256
        };

        let mut buffer = vec![0; len];
        let mut length = 0;
        gl.GetActiveUniformBlockName(
            program.0.get(),
            uniform_block_index,
            buffer.len() as _,
            &mut length,
            buffer.as_mut_ptr(),
        );

        if length > 0 {
            assert_eq!(
                std::mem::size_of::<u8>(),
                std::mem::size_of::<native_gl::GLchar>(),
                "This operation is only safe in systems in which the length of \
                a GLchar is the same as that of an u8"
            );
            assert_eq!(
                std::mem::align_of::<u8>(),
                std::mem::align_of::<native_gl::GLchar>(),
                "This operation is only safe in systems in which the alignment \
                of a GLchar is the same as that of an u8"
            );
            let buffer = std::slice::from_raw_parts(
                buffer.as_ptr() as *const u8,
                (length as usize + 1).min(buffer.len()),
            );

            let name = CStr::from_bytes_with_nul(&buffer[..])
                .unwrap()
                .to_str()
                .unwrap()
                .to_owned();

            name
        } else {
            String::from("")
        }
    }

    unsafe fn max_shader_compiler_threads(&self, count: u32) {
        let gl = &self.raw;
        if gl.MaxShaderCompilerThreadsKHR_is_loaded() {
            gl.MaxShaderCompilerThreadsKHR(count);
        } else {
            gl.MaxShaderCompilerThreadsARB(count);
        }
    }

    unsafe fn multi_draw_arrays_instanced(
        &self,
        mode: u32,
        firsts_list: &[i32],
        firsts_offset: u32,
        counts_list: &[i32],
        counts_offset: u32,
        instance_counts_list: &[i32],
        instance_counts_offset: u32,
        drawcount: i32,
    ) {
        let gl = &self.raw;

        let mut data: Vec<u8> = Vec::new();

        for draw_index in 0usize..drawcount as usize {
            let firsts_index = draw_index + firsts_offset as usize;
            let counts_index = draw_index + counts_offset as usize;
            let instance_counts_index = draw_index + instance_counts_offset as usize;

            let first = firsts_list[firsts_index];
            let count = counts_list[counts_index];
            let instance_count = instance_counts_list[instance_counts_index];

            let command = DrawArraysIndirectCommand::new(
                count as u32,
                instance_count as u32,
                first as u32,
                0 // TODO: what value goes here???
            );

            data.extend(&command.to_bytes());
        }

        gl.MultiDrawArraysIndirect(
            mode,
            data.as_ptr() as *const std::ffi::c_void,
            drawcount,
            0, // docs say a stride of 0 means that the commands are tightly packed
        );
    }
}

struct DrawArraysIndirectCommand {
    count: u32,
    instance_count: u32,
    first: u32,
    base_instance: u32,
}

impl DrawArraysIndirectCommand {
    fn new(count: u32, instance_count: u32, first: u32, base_instance: u32) -> Self {
        Self {
            count,
            instance_count,
            first,
            base_instance,
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.count.to_ne_bytes());
        bytes.extend_from_slice(&self.instance_count.to_ne_bytes());
        bytes.extend_from_slice(&self.first.to_ne_bytes());
        bytes.extend_from_slice(&self.base_instance.to_ne_bytes());
        bytes
    }
}
