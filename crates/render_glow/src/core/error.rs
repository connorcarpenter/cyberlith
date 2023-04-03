use thiserror::Error;

///
/// Error in the [core](crate::core) module.
///
#[derive(Debug, Error)]
#[allow(missing_docs)]
pub enum CoreError {
    #[error("failed creating context with error: {0}")]
    ContextCreation(String),
    #[error("failed rendering with error: {0}")]
    ContextError(String),
    #[error("failed compiling {0} shader: {1}\n{2}")]
    ShaderCompilation(String, String, String),
    #[error("failed to link shader program: {0}")]
    ShaderLink(String),
}
