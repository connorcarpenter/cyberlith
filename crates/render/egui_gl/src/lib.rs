//! [`egui`] bindings for [`gl`](https://github.com/grovesNL/gl).
//!
//! The main type you want to look at is [`Painter`].
//!
//! If you are writing an app, you may want to look at [`eframe`](https://docs.rs/eframe) instead.
//!
//! ## Feature flags
#![cfg_attr(feature = "document-features", doc = document_features::document_features!())]
//!

#![allow(clippy::float_cmp)]
#![allow(clippy::manual_range_contains)]

pub mod painter;
pub use gl;
pub use painter::{CallbackFn, Painter, PainterError};
mod misc_util;
mod shader_version;
mod vao;

pub use shader_version::ShaderVersion;

#[cfg(all(not(target_arch = "wasm32"), feature = "winit"))]
pub mod winit;
#[cfg(all(not(target_arch = "wasm32"), feature = "winit"))]
pub use winit::*;

/// Check for OpenGL error and report it using `logging::error`.
///
/// Only active in debug builds!
///
/// ``` no_run
/// # let gl_context = todo!();
/// use egui_gl::check_for_gl_error;
/// check_for_gl_error!(gl_context);
/// check_for_gl_error!(gl_context, "during painting");
/// ```
#[macro_export]
macro_rules! check_for_gl_error {
    ($gl: expr) => {{
        if cfg!(debug_assertions) {
            $crate::check_for_gl_error_impl($gl, file!(), line!(), "")
        }
    }};
    ($gl: expr, $context: literal) => {{
        if cfg!(debug_assertions) {
            $crate::check_for_gl_error_impl($gl, file!(), line!(), $context)
        }
    }};
}

/// Check for OpenGL error and report it using `logging::error`.
///
/// WARNING: slow! Only use during setup!
///
/// ``` no_run
/// # let gl_context = todo!();
/// use egui_gl::check_for_gl_error_even_in_release;
/// check_for_gl_error_even_in_release!(gl_context);
/// check_for_gl_error_even_in_release!(gl_context, "during painting");
/// ```
#[macro_export]
macro_rules! check_for_gl_error_even_in_release {
    ($gl: expr) => {{
        $crate::check_for_gl_error_impl($gl, file!(), line!(), "")
    }};
    ($gl: expr, $context: literal) => {{
        $crate::check_for_gl_error_impl($gl, file!(), line!(), $context)
    }};
}

#[doc(hidden)]
pub fn check_for_gl_error_impl(gl: &gl::Context, file: &str, line: u32, context: &str) {
    use gl::HasContext as _;
    #[allow(unsafe_code)]
    let error_code = unsafe { gl.get_error() };
    if error_code != gl::NO_ERROR {
        let error_str = match error_code {
            gl::INVALID_ENUM => "GL_INVALID_ENUM",
            gl::INVALID_VALUE => "GL_INVALID_VALUE",
            gl::INVALID_OPERATION => "GL_INVALID_OPERATION",
            gl::STACK_OVERFLOW => "GL_STACK_OVERFLOW",
            gl::STACK_UNDERFLOW => "GL_STACK_UNDERFLOW",
            gl::OUT_OF_MEMORY => "GL_OUT_OF_MEMORY",
            gl::INVALID_FRAMEBUFFER_OPERATION => "GL_INVALID_FRAMEBUFFER_OPERATION",
            gl::CONTEXT_LOST => "GL_CONTEXT_LOST",
            0x8031 => "GL_TABLE_TOO_LARGE1",
            0x9242 => "CONTEXT_LOST_WEBGL",
            _ => "<unknown>",
        };

        if context.is_empty() {
            logging::error!(
                "GL error, at {}:{}: {} (0x{:X}). Please file a bug at https://github.com/emilk/egui/issues",
                file,
                line,
                error_str,
                error_code,
            );
        } else {
            logging::error!(
                "GL error, at {}:{} ({}): {} (0x{:X}). Please file a bug at https://github.com/emilk/egui/issues",
                file,
                line,
                context,
                error_str,
                error_code,
            );
        }
    }
}

// ---------------------------------------------------------------------------

mod profiling_scopes {
    #![allow(unused_macros)]
    #![allow(unused_imports)]

    /// Profiling macro for feature "puffin"
    macro_rules! profile_function {
        ($($arg: tt)*) => {
            #[cfg(feature = "puffin")]
            #[cfg(not(target_arch = "wasm32"))] // Disabled on web because of the coarse 1ms clock resolution there.
            puffin::profile_function!($($arg)*);
        };
    }
    pub(crate) use profile_function;

    /// Profiling macro for feature "puffin"
    macro_rules! profile_scope {
        ($($arg: tt)*) => {
            #[cfg(feature = "puffin")]
            #[cfg(not(target_arch = "wasm32"))] // Disabled on web because of the coarse 1ms clock resolution there.
            puffin::profile_scope!($($arg)*);
        };
    }
    pub(crate) use profile_scope;
}

#[allow(unused_imports)]
pub(crate) use profiling_scopes::*;
