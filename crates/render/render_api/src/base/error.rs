/// A result for this crate.
pub type Result<T> = std::result::Result<T, Error>;

use thiserror::Error;
///
/// Error from this crate.
///
#[derive(Error, Debug)]
#[allow(missing_docs)]
pub enum Error {
    #[error("{0} buffer length must be {1}, actual length is {2}")]
    InvalidBufferLength(String, usize, usize),
    #[error("the number of indices must be divisable by 3, actual count is {0}")]
    InvalidNumberOfIndices(usize),
    #[error("the max index {0} must be less than the number of vertices {1}")]
    InvalidIndices(usize, usize),
    #[error("the transformation matrix cannot be inverted and is therefore invalid")]
    FailedInvertingTransformationMatrix,

    #[cfg(not(target_arch = "wasm32"))]
    #[error("io error")]
    IO(#[from] std::io::Error),
    #[error("the .vol file contain wrong data size")]
    VolCorruptData,
    #[cfg(not(target_arch = "wasm32"))]
    #[error("error while loading the file {0}: {1}")]
    FailedLoading(String, std::io::Error),
    #[error("tried to use {0} which was not loaded or otherwise added to the raw assets")]
    NotLoaded(String),
    #[error("the feature {0} is needed")]
    FeatureMissing(String),
    #[error("failed to deserialize the file {0}")]
    FailedDeserialize(String),
    #[error("failed to serialize the file {0}")]
    FailedSerialize(String),
    #[error("failed to find {0} in the file {1}")]
    FailedConvertion(String, String),
}
