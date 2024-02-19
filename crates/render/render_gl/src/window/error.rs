use thiserror::Error;

use crate::core::CoreError;

///
/// Error associated with a window.
///
#[cfg(not(target_arch = "wasm32"))]
#[derive(Error, Debug)]
#[allow(missing_docs)]
pub enum WindowError {
    #[error("glutin error")]
    GlutinError(#[from] glutin::error::Error),
    #[error("winit error")]
    WinitError(#[from] winit::error::OsError),
    #[error("winit event loop error")]
    WinitEventLoopError(#[from] winit::error::EventLoopError),
    #[error("winit handle error")]
    WinitHandleError(#[from] raw_window_handle::HandleError),
    #[error("error in three-d")]
    ThreeDError(#[from] CoreError),
    #[error("the number of MSAA samples must be a power of two")]
    InvalidNumberOfMSAASamples,
    #[error("it's not possible to create a graphics context/surface with the given settings")]
    SurfaceCreationError,
}

///
/// Error associated with a window.
///
#[cfg(target_arch = "wasm32")]
#[derive(Error, Debug)]
#[allow(missing_docs)]
pub enum WindowError {
    #[error("failed to create a new winit window")]
    WinitError(#[from] winit::error::OsError),
    #[error("winit event loop error")]
    WinitEventLoopError(#[from] winit::error::EventLoopError),
    #[error("failed creating a new window")]
    WindowCreation,
    #[error("unable to get document from canvas")]
    DocumentMissing,
    #[error("unable to convert canvas to html canvas: {0}")]
    CanvasConvertFailed(String),
    #[error("unable to get webgl2 context for the given canvas, maybe the browser doesn't support WebGL2{0}")]
    WebGL2NotSupported(String),
    #[error("unable to get EXT_color_buffer_float extension for the given canvas, maybe the browser doesn't support EXT_color_buffer_float: {0}")]
    ColorBufferFloatNotSupported(String),
    #[error("unable to get OES_texture_float extension for the given canvas, maybe the browser doesn't support OES_texture_float: {0}")]
    OESTextureFloatNotSupported(String),
    #[error("unable to get OES_standard_derivatives extension for the given canvas, maybe the browser doesn't support OES_standard_derivatives: {0}")]
    OESStandardDerivativesNotSupported(String),
    #[error("error in three-d")]
    ThreeDError(#[from] CoreError),
}
