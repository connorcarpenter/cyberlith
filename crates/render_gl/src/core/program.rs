use std::{collections::HashMap, sync::RwLock};

use gl::{DrawArraysIndirectCommand, HasContext};

use render_api::components::Viewport;

use crate::core::*;

///
/// A shader program consisting of a programmable vertex shader followed by a programmable fragment shader.
/// Functionality includes transferring per vertex data to the vertex shader (see the use_attribute functionality)
/// and transferring uniform data to both shader stages (see the use_uniform and use_texture functionality)
/// and execute the shader program (see the draw functionality).
///
pub struct Program {
    id: gl::Program,
    attributes: HashMap<String, u32>,
    textures: RwLock<HashMap<String, u32>>,
    uniforms: HashMap<String, gl::UniformLocation>,
    uniform_blocks: RwLock<HashMap<String, (u32, u32)>>,
}

impl Program {
    ///
    /// Creates a new shader program from the given vertex and fragment glsl shader source.
    ///
    pub fn from_source(
        vertex_shader_source: &str,
        fragment_shader_source: &str,
    ) -> Result<Self, CoreError> {
        unsafe {
            let context = Context::get();
            let vert_shader = context
                .create_shader(gl::VERTEX_SHADER)
                .expect("Failed creating vertex shader");
            let frag_shader = context
                .create_shader(gl::FRAGMENT_SHADER)
                .expect("Failed creating fragment shader");

            let header: &str = if context.version().is_embedded {
                "#version 300 es
                    #ifdef GL_FRAGMENT_PRECISION_HIGH
                        precision highp float;
                        precision highp int;
                        precision highp sampler2DArray;
                        precision highp sampler3D;
                    #else
                        precision mediump float;
                        precision mediump int;
                        precision mediump sampler2DArray;
                        precision mediump sampler3D;
                    #endif\n"
            } else {
                "#version 330 core\n"
            };
            let vertex_shader_source = format!("{}{}", header, vertex_shader_source);
            let fragment_shader_source = format!("{}{}", header, fragment_shader_source);

            context.shader_source(vert_shader, &vertex_shader_source);
            context.shader_source(frag_shader, &fragment_shader_source);
            context.compile_shader(vert_shader);
            context.compile_shader(frag_shader);

            let id = context.create_program().expect("Failed creating program");
            context.attach_shader(id, vert_shader);
            context.attach_shader(id, frag_shader);
            context.link_program(id);

            if !context.get_program_link_status(id) {
                let log = context.get_shader_info_log(vert_shader);
                if !log.is_empty() {
                    Err(CoreError::ShaderCompilation(
                        "vertex".to_string(),
                        log,
                        vertex_shader_source,
                    ))?;
                }
                let log = context.get_shader_info_log(frag_shader);
                if !log.is_empty() {
                    Err(CoreError::ShaderCompilation(
                        "fragment".to_string(),
                        log,
                        fragment_shader_source,
                    ))?;
                }
                let log = context.get_program_info_log(id);
                if !log.is_empty() {
                    Err(CoreError::ShaderLink(log))?;
                }
                unreachable!();
            }

            context.detach_shader(id, vert_shader);
            context.detach_shader(id, frag_shader);
            context.delete_shader(vert_shader);
            context.delete_shader(frag_shader);

            // Init vertex attributes
            let num_attribs = context.get_active_attributes(id);
            let mut attributes = HashMap::new();
            for i in 0..num_attribs {
                if let Some(gl::ActiveAttribute { name, .. }) =
                    context.get_active_attribute(id, i)
                {
                    let location = context.get_attrib_location(id, &name).unwrap_or_else(|| {
                        panic!("Could not get the location of uniform {}", name)
                    });
                    /*println!(
                        "Attribute location: {}, name: {}, type: {}, size: {}",
                        location, name, atype, size
                    );*/
                    attributes.insert(name, location);
                }
            }

            // Init uniforms
            let num_uniforms = context.get_active_uniforms(id);
            let mut uniforms = HashMap::new();
            for i in 0..num_uniforms {
                if let Some(gl::ActiveUniform { name, .. }) = context.get_active_uniform(id, i) {
                    if let Some(location) = context.get_uniform_location(id, &name) {
                        let name = name.split('[').collect::<Vec<_>>()[0].to_string();
                        /*println!(
                            "Uniform location: {:?}, name: {}, type: {}, size: {}",
                            location, name, utype, size
                        );*/
                        uniforms.insert(name, location);
                    }
                }
            }

            Ok(Program {
                id,
                attributes,
                uniforms,
                uniform_blocks: RwLock::new(HashMap::new()),
                textures: RwLock::new(HashMap::new()),
            })
        }
    }

    ///
    /// Send the given uniform data to this shader program and associate it with the given named variable.
    /// The glsl shader variable must be of type `uniform int` if the data is an integer, `uniform vec2` if it is of type [Vec2] etc.
    /// The uniform variable is uniformly available across all processing of vertices and fragments.
    ///
    /// # Panic
    /// Will panic if the uniform is not defined or not used in the shader code.
    /// In the latter case the variable is removed by the shader compiler.
    ///
    pub fn use_uniform<T: UniformDataType>(&self, name: &str, data: T) {
        let location = self.get_uniform_location(name);
        T::send_uniform(location, &[data]);
        self.unuse_program();
    }

    ///
    /// Calls [Self::use_uniform] if [Self::requires_uniform] returns true.
    ///
    pub fn use_uniform_if_required<T: UniformDataType>(&self, name: &str, data: T) {
        if self.requires_uniform(name) {
            self.use_uniform(name, data);
        }
    }

    ///
    /// Send the given array of uniform data to this shader program and associate it with the given named variable.
    /// The glsl shader variable must be of same type and length as the data, so if the data is an array of three [Vec2], the variable must be `uniform vec2[3]`.
    /// The uniform variable is uniformly available across all processing of vertices and fragments.
    ///
    /// # Panic
    /// Will panic if the uniform is not defined in the shader code or not used.
    /// In the latter case the variable is removed by the shader compiler.
    ///
    pub fn use_uniform_array<T: UniformDataType>(&self, name: &str, data: &[T]) {
        let location = self.get_uniform_location(name);
        T::send_uniform(location, data);
        self.unuse_program();
    }

    fn get_uniform_location(&self, name: &str) -> &gl::UniformLocation {
        self.use_program();
        self.uniforms.get(name).unwrap_or_else(|| {
            panic!(
                "the uniform {} is sent to the shader but not defined or never used",
                name
            )
        })
    }

    ///
    /// Use the given [GpuTexture2D] in this shader program and associate it with the given named variable.
    /// The glsl shader variable must be of type `uniform sampler2D` and can only be accessed in the fragment shader.
    ///
    /// # Panic
    /// Will panic if the texture is not defined in the shader code or not used.
    /// In the latter case the variable is removed by the shader compiler.
    ///
    pub fn use_texture(&self, name: &str, texture: &GpuTexture2D) {
        self.use_texture_internal(name);
        texture.bind();
    }

    ///
    /// Use the given [GpuDepthTexture2D] in this shader program and associate it with the given named variable.
    /// The glsl shader variable must be of type `uniform sampler2D` and can only be accessed in the fragment shader.
    ///
    /// # Panic
    /// Will panic if the texture is not defined in the shader code or not used.
    /// In the latter case the variable is removed by the shader compiler.
    ///
    pub fn use_depth_texture(&self, name: &str, texture: &GpuDepthTexture2D) {
        self.use_texture_internal(name);
        texture.bind();
    }

    fn use_texture_internal(&self, name: &str) -> u32 {
        if !self.textures.read().unwrap().contains_key(name) {
            let mut map = self.textures.write().unwrap();
            let index = map.len() as u32;
            map.insert(name.to_owned(), index);
        };
        let index = *self.textures.read().unwrap().get(name).unwrap();
        self.use_uniform(name, index as i32);
        unsafe {
            Context::get().active_texture(gl::TEXTURE0 + index);
        }
        index
    }

    ///
    /// Use the given [UniformBuffer] in this shader program and associate it with the given named variable.
    ///
    pub fn use_uniform_block(&self, name: &str, buffer: &UniformBuffer) {
        let context = Context::get();
        if !self.uniform_blocks.read().unwrap().contains_key(name) {
            let mut map = self.uniform_blocks.write().unwrap();
            let location = unsafe {
                context
                    .get_uniform_block_index(self.id, name)
                    .unwrap_or_else(|| panic!("the uniform block {} is sent to the shader but not defined or never used",
                        name))
            };
            let index = map.len() as u32;
            map.insert(name.to_owned(), (location, index));
        };
        let (location, index) = *self.uniform_blocks.read().unwrap().get(name).unwrap();
        unsafe {
            context.uniform_block_binding(self.id, location, index);
            buffer.bind(index);
            context.bind_buffer(gl::UNIFORM_BUFFER, None);
        }
    }

    ///
    /// Uses the given [VertexBuffer] data in this shader program and associates it with the given named variable.
    /// Each value in the buffer is used when rendering one vertex using the [Program::draw_arrays] methods.
    /// Therefore the buffer must contain the same number of values as the number of vertices specified in those draw calls.
    ///
    /// # Panic
    /// Will panic if the attribute is not defined in the shader code or not used.
    /// In the latter case the variable is removed by the shader compiler.
    ///
    pub fn use_vertex_attribute(&self, name: &str, buffer: &VertexBuffer) {
        let context = Context::get();
        if buffer.count() > 0 {
            buffer.bind();
            let loc = self.location(name);
            unsafe {
                context.bind_vertex_array(Some(context.vao()));
                context.enable_vertex_attrib_array(loc);
                if buffer.data_type() == gl::UNSIGNED_SHORT
                    || buffer.data_type() == gl::SHORT
                    || buffer.data_type() == gl::UNSIGNED_INT
                    || buffer.data_type() == gl::INT
                {
                    context.vertex_attrib_pointer_i32(
                        loc,
                        buffer.data_size() as i32,
                        buffer.data_type(),
                        0,
                        0,
                    );
                } else {
                    context.vertex_attrib_pointer_f32(
                        loc,
                        buffer.data_size() as i32,
                        buffer.data_type(),
                        false,
                        0,
                        0,
                    );
                }
                context.vertex_attrib_divisor(loc, 0);
                context.bind_buffer(gl::ARRAY_BUFFER, None);
            }
            self.unuse_program();
        }
    }

    pub fn use_vertex_attribute_if_required(&self, name: &str, buffer: &VertexBuffer) {
        if self.requires_attribute(name) {
            self.use_vertex_attribute(name, buffer);
        }
    }

    ///
    /// Uses the given [InstanceBuffer] data in this shader program and associates it with the given named variable.
    /// Each value in the buffer is used when rendering one instance using the [Program::draw_arrays_instanced] methods.
    /// Therefore the buffer must contain the same number of values as the number of instances specified in those draw calls.
    ///
    /// # Panic
    /// Will panic if the attribute is not defined in the shader code or not used.
    /// In the latter case the variable is removed by the shader compiler.
    ///
    pub fn use_instance_attribute(&self, name: &str, buffer: &InstanceBuffer) {
        let context = Context::get();
        if buffer.count() > 0 {
            buffer.bind();
            let loc = self.location(name);
            unsafe {
                context.bind_vertex_array(Some(context.vao()));
                context.enable_vertex_attrib_array(loc);
                if buffer.data_type() == gl::UNSIGNED_SHORT
                    || buffer.data_type() == gl::SHORT
                    || buffer.data_type() == gl::UNSIGNED_INT
                    || buffer.data_type() == gl::INT
                {
                    context.vertex_attrib_pointer_i32(
                        loc,
                        buffer.data_size() as i32,
                        buffer.data_type(),
                        0,
                        0,
                    );
                } else {
                    context.vertex_attrib_pointer_f32(
                        loc,
                        buffer.data_size() as i32,
                        buffer.data_type(),
                        false,
                        0,
                        0,
                    );
                }
                context.vertex_attrib_divisor(loc, 1);
                context.bind_buffer(gl::ARRAY_BUFFER, None);
            }
            self.unuse_program();
        }
    }

    ///
    /// Draws `count` number of triangles with the given render states and viewport using this shader program.
    /// Requires that all attributes and uniforms have been defined using the use_attribute and use_uniform methods.
    /// Assumes that the data for the three vertices in a triangle is defined contiguous in each vertex buffer.
    ///
    pub fn draw_arrays(&self, render_states: RenderStates, viewport: Viewport, count: u32) {
        let context = Context::get();
        context.set_viewport(viewport);
        context.set_render_states(render_states);
        self.use_program();
        unsafe {
            context.draw_arrays(gl::TRIANGLES, 0, count as i32);
            for location in self.attributes.values() {
                context.disable_vertex_attrib_array(*location);
            }
            context.bind_vertex_array(None);
        }
        self.unuse_program();

        #[cfg(debug_assertions)]
        context
            .error_check()
            .expect("Unexpected rendering error occured")
    }

    ///
    /// Same as [Program::draw_arrays] except it renders 'instance_count' instances of the same set of triangles.
    /// Use the [Program::use_instance_attribute], method to send unique data for each instance to the shader.
    ///
    pub fn draw_arrays_instanced(
        &self,
        render_states: RenderStates,
        viewport: Viewport,
        count: u32,
        instance_count: u32,
    ) {
        let context = Context::get();
        context.set_viewport(viewport);
        context.set_render_states(render_states);
        self.use_program();
        unsafe {
            context.draw_arrays_instanced(gl::TRIANGLES, 0, count as i32, instance_count as i32);
            context.bind_buffer(gl::ELEMENT_ARRAY_BUFFER, None);
            for location in self.attributes.values() {
                context.disable_vertex_attrib_array(*location);
            }
            context.bind_vertex_array(None);
        }
        self.unuse_program();

        #[cfg(debug_assertions)]
        context
            .error_check()
            .expect("Unexpected rendering error occured")
    }

    pub fn multi_draw_arrays_indirect(
        &self,
        render_states: RenderStates,
        viewport: Viewport,
        draw_commands: Vec<DrawArraysIndirectCommand>,
    ) {
        let context = Context::get();
        context.set_viewport(viewport);
        context.set_render_states(render_states);
        self.use_program();
        unsafe {
            context.multi_draw_arrays_instanced(gl::TRIANGLES, draw_commands);
            context.bind_buffer(gl::ELEMENT_ARRAY_BUFFER, None);
            for location in self.attributes.values() {
                context.disable_vertex_attrib_array(*location);
            }
            context.bind_vertex_array(None);
        }
        self.unuse_program();

        #[cfg(debug_assertions)]
        context
            .error_check()
            .expect("Unexpected rendering error occured")
    }

    ///
    /// Returns true if this program uses the uniform with the given name.
    ///
    pub fn requires_uniform(&self, name: &str) -> bool {
        self.uniforms.contains_key(name)
    }

    ///
    /// Returns true if this program uses the attribute with the given name.
    ///
    pub fn requires_attribute(&self, name: &str) -> bool {
        self.attributes.contains_key(name)
    }

    fn location(&self, name: &str) -> u32 {
        self.use_program();
        *self.attributes.get(name).unwrap_or_else(|| {
            panic!(
                "the attribute {} is sent to the shader but not defined or never used",
                name
            )
        })
    }

    fn use_program(&self) {
        unsafe {
            Context::get().use_program(Some(self.id));
        }
    }

    fn unuse_program(&self) {
        unsafe {
            Context::get().use_program(None);
        }
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        unsafe {
            Context::get().delete_program(self.id);
        }
    }
}
