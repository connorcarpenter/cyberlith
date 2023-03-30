use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;

use super::*;
use glow::HasContext;

///
/// Contains the low-level OpenGL/WebGL graphics context as well as other "global" variables.
/// Implements Deref with the low-level graphics context as target, so you can call low-level functionality
/// directly on this struct. Use the [context](glow) module to get access to low-level constants and structs.
///
#[derive(Clone)]
pub struct Context {
    context: Arc<glow::Context>,
    pub(super) vao: glow::VertexArray,
    programs: Arc<RwLock<HashMap<(String, String), Program>>>,
}

impl Context {
    ///
    /// Creates a new mid-level context, used in this [core](crate::core) module, from a low-level OpenGL/WebGL context from the [context](glow) module.
    /// This should only be called directly if you are creating a low-level context yourself (ie. not using the features in the [window](crate::window) module).
    /// Since the content in the [context](glow) module is just a re-export of [glow](https://crates.io/crates/glow),
    /// you can also call this method with a reference counter to a glow context created using glow and not the re-export in [context](glow).
    ///
    pub fn from_gl_context(context: Arc<glow::Context>) -> Result<Self, CoreError> {
        unsafe {
            if !context.version().is_embedded {
                // Enable seamless cube map textures - not available on OpenGL ES and WebGL
                context.enable(glow::TEXTURE_CUBE_MAP_SEAMLESS);
            }
            context.pixel_store_i32(glow::UNPACK_ALIGNMENT, 1);
            context.pixel_store_i32(glow::PACK_ALIGNMENT, 1);
        };
        let c = unsafe {
            // Create one Vertex Array Object which is then reused all the time.
            let vao = context
                .create_vertex_array()
                .map_err(CoreError::ContextCreation)?;
            Self {
                context,
                vao,
                programs: Arc::new(RwLock::new(HashMap::new())),
            }
        };
        Ok(c)
    }

    ///
    /// Compiles a [Program] with the given vertex and fragment shader source and stores it for later use.
    /// If it has already been created, then it is just returned.
    ///
    pub fn program(
        &self,
        vertex_shader_source: String,
        fragment_shader_source: String,
        callback: impl FnOnce(&Program),
    ) -> Result<(), CoreError> {
        let key = (vertex_shader_source, fragment_shader_source);
        let mut programs = self.programs.write().unwrap();
        if let Some(program) = programs.get(&key) {
            callback(program);
        } else {
            let program = Program::from_source(self, &key.0, &key.1)?;
            callback(&program);
            programs.insert(key, program);
        }
        Ok(())
    }

    ///
    /// Set the scissor test for this context (see [ScissorBox]).
    ///
    pub fn set_scissor(&self, scissor_box: ScissorBox) {
        unsafe {
            if scissor_box.width > 0 && scissor_box.height > 0 {
                self.enable(glow::SCISSOR_TEST);
                self.scissor(
                    scissor_box.x,
                    scissor_box.y,
                    scissor_box.width as i32,
                    scissor_box.height as i32,
                );
            } else {
                self.disable(glow::SCISSOR_TEST);
            }
        }
    }

    ///
    /// Set the viewport for this context (See [Viewport]).
    ///
    pub fn set_viewport(&self, viewport: Viewport) {
        unsafe {
            self.viewport(
                viewport.x,
                viewport.y,
                viewport.width as i32,
                viewport.height as i32,
            );
        }
    }

    ///
    /// Set the face culling for this context (see [Cull]).
    ///
    pub fn set_cull(&self, cull: Cull) {
        unsafe {
            match cull {
                Cull::None => {
                    self.disable(glow::CULL_FACE);
                }
                Cull::Back => {
                    self.enable(glow::CULL_FACE);
                    self.cull_face(glow::BACK);
                }
                Cull::Front => {
                    self.enable(glow::CULL_FACE);
                    self.cull_face(glow::FRONT);
                }
                Cull::FrontAndBack => {
                    self.enable(glow::CULL_FACE);
                    self.cull_face(glow::FRONT_AND_BACK);
                }
            }
        }
    }

    ///
    /// Set the write mask for this context (see [WriteMask]).
    ///
    pub fn set_write_mask(&self, write_mask: WriteMask) {
        unsafe {
            self.color_mask(
                write_mask.red,
                write_mask.green,
                write_mask.blue,
                write_mask.alpha,
            );
            self.depth_mask(write_mask.depth);
        }
    }

    ///
    /// Set the depth test for this context (see [DepthTest]).
    ///
    pub fn set_depth_test(&self, depth_test: DepthTest) {
        unsafe {
            self.enable(glow::DEPTH_TEST);
            match depth_test {
                DepthTest::Never => {
                    self.depth_func(glow::NEVER);
                }
                DepthTest::Less => {
                    self.depth_func(glow::LESS);
                }
                DepthTest::Equal => {
                    self.depth_func(glow::EQUAL);
                }
                DepthTest::LessOrEqual => {
                    self.depth_func(glow::LEQUAL);
                }
                DepthTest::Greater => {
                    self.depth_func(glow::GREATER);
                }
                DepthTest::NotEqual => {
                    self.depth_func(glow::NOTEQUAL);
                }
                DepthTest::GreaterOrEqual => {
                    self.depth_func(glow::GEQUAL);
                }
                DepthTest::Always => {
                    self.depth_func(glow::ALWAYS);
                }
            }
        }
    }

    ///
    /// Set the blend state for this context (see [Blend]).
    ///
    pub fn set_blend(&self, blend: Blend) {
        unsafe {
            if let Blend::Enabled {
                source_rgb_multiplier,
                source_alpha_multiplier,
                destination_rgb_multiplier,
                destination_alpha_multiplier,
                rgb_equation,
                alpha_equation,
            } = blend
            {
                self.enable(glow::BLEND);
                self.blend_func_separate(
                    Self::blend_const_from_multiplier(source_rgb_multiplier),
                    Self::blend_const_from_multiplier(destination_rgb_multiplier),
                    Self::blend_const_from_multiplier(source_alpha_multiplier),
                    Self::blend_const_from_multiplier(destination_alpha_multiplier),
                );
                self.blend_equation_separate(
                    Self::blend_const_from_equation(rgb_equation),
                    Self::blend_const_from_equation(alpha_equation),
                );
            } else {
                self.disable(glow::BLEND);
            }
        }
    }

    fn blend_const_from_multiplier(multiplier: BlendMultiplierType) -> u32 {
        match multiplier {
            BlendMultiplierType::Zero => glow::ZERO,
            BlendMultiplierType::One => glow::ONE,
            BlendMultiplierType::SrcColor => glow::SRC_COLOR,
            BlendMultiplierType::OneMinusSrcColor => glow::ONE_MINUS_SRC_COLOR,
            BlendMultiplierType::DstColor => glow::DST_COLOR,
            BlendMultiplierType::OneMinusDstColor => glow::ONE_MINUS_DST_COLOR,
            BlendMultiplierType::SrcAlpha => glow::SRC_ALPHA,
            BlendMultiplierType::OneMinusSrcAlpha => glow::ONE_MINUS_SRC_ALPHA,
            BlendMultiplierType::DstAlpha => glow::DST_ALPHA,
            BlendMultiplierType::OneMinusDstAlpha => glow::ONE_MINUS_DST_ALPHA,
            BlendMultiplierType::SrcAlphaSaturate => glow::SRC_ALPHA_SATURATE,
        }
    }
    fn blend_const_from_equation(equation: BlendEquationType) -> u32 {
        match equation {
            BlendEquationType::Add => glow::FUNC_ADD,
            BlendEquationType::Subtract => glow::FUNC_SUBTRACT,
            BlendEquationType::ReverseSubtract => glow::FUNC_REVERSE_SUBTRACT,
            BlendEquationType::Min => glow::MIN,
            BlendEquationType::Max => glow::MAX,
        }
    }

    ///
    /// Set the render states for this context (see [RenderStates]).
    ///
    pub fn set_render_states(&self, render_states: RenderStates) {
        self.set_cull(render_states.cull);
        self.set_write_mask(render_states.write_mask);
        if !render_states.write_mask.depth && render_states.depth_test == DepthTest::Always {
            unsafe { self.disable(glow::DEPTH_TEST) }
        } else {
            self.set_depth_test(render_states.depth_test);
        }
        self.set_blend(render_states.blend);
    }

    ///
    /// Returns an error if an GPU-side error has happened while rendering which can be used to check for errors while developing.
    /// Can also be used in production to handle unexpected rendering errors, but do not call it too often to avoid performance problems.
    ///
    pub fn error_check(&self) -> Result<(), CoreError> {
        self.framebuffer_check()?;
        unsafe {
            let e = self.get_error();
            if e != glow::NO_ERROR {
                Err(CoreError::ContextError(
                    match e {
                        glow::INVALID_ENUM => "Invalid enum",
                        glow::INVALID_VALUE => "Invalid value",
                        glow::INVALID_OPERATION => "Invalid operation",
                        glow::INVALID_FRAMEBUFFER_OPERATION => "Invalid framebuffer operation",
                        glow::OUT_OF_MEMORY => "Out of memory",
                        glow::STACK_OVERFLOW => "Stack overflow",
                        glow::STACK_UNDERFLOW => "Stack underflow",
                        _ => "Unknown",
                    }
                    .to_string(),
                ))?;
            }
        }
        Ok(())
    }

    fn framebuffer_check(&self) -> Result<(), CoreError> {
        unsafe {
            match self.check_framebuffer_status(glow::FRAMEBUFFER) {
                glow::FRAMEBUFFER_COMPLETE => Ok(()),
                glow::FRAMEBUFFER_INCOMPLETE_ATTACHMENT => Err(CoreError::ContextError(
                    "FRAMEBUFFER_INCOMPLETE_ATTACHMENT".to_string(),
                )),
                glow::FRAMEBUFFER_INCOMPLETE_DRAW_BUFFER => Err(CoreError::ContextError(
                    "FRAMEBUFFER_INCOMPLETE_DRAW_BUFFER".to_string(),
                )),
                glow::FRAMEBUFFER_INCOMPLETE_MISSING_ATTACHMENT => Err(CoreError::ContextError(
                    "FRAMEBUFFER_INCOMPLETE_MISSING_ATTACHMENT".to_string(),
                )),
                glow::FRAMEBUFFER_UNSUPPORTED => Err(CoreError::ContextError(
                    "FRAMEBUFFER_UNSUPPORTED".to_string(),
                )),
                glow::FRAMEBUFFER_UNDEFINED => {
                    Err(CoreError::ContextError("FRAMEBUFFER_UNDEFINED".to_string()))
                }
                glow::FRAMEBUFFER_INCOMPLETE_READ_BUFFER => Err(CoreError::ContextError(
                    "FRAMEBUFFER_INCOMPLETE_READ_BUFFER".to_string(),
                )),
                glow::FRAMEBUFFER_INCOMPLETE_MULTISAMPLE => Err(CoreError::ContextError(
                    "FRAMEBUFFER_INCOMPLETE_MULTISAMPLE".to_string(),
                )),
                glow::FRAMEBUFFER_INCOMPLETE_LAYER_TARGETS => Err(CoreError::ContextError(
                    "FRAMEBUFFER_INCOMPLETE_LAYER_TARGETS".to_string(),
                )),
                _ => Err(CoreError::ContextError(
                    "Unknown framebuffer error".to_string(),
                )),
            }?;
        }
        Ok(())
    }
}

impl std::fmt::Debug for Context {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut d = f.debug_struct("Context");
        d.field("programs", &self.programs.read().unwrap().len());
        d.finish()
    }
}

impl std::ops::Deref for Context {
    type Target = Arc<glow::Context>;
    fn deref(&self) -> &Self::Target {
        &self.context
    }
}
