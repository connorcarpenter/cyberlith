#![cfg_attr(docsrs, feature(doc_cfg))]
//#![warn(clippy::all)]
#![warn(missing_docs)]

//!
//! A set of common assets that are useful when doing graphics, for example [TriMesh], [Texture2D] or [PbrMaterial].
//! These assets can be loaded using the [io] module or constructed manually.
//! When in memory, the assets can be for example be
//! - visualised, for example using the [three-d](https://github.com/asny/three-d) crate or in a CPU ray tracer
//! - imported into a rust-based game engine
//! - edited and saved again
//!

mod camera;
mod geometry;
mod material;
mod prelude;
mod texture;

pub use camera::*;
pub use geometry::*;
pub use material::*;
pub use prelude::*;
pub use texture::*;

///
/// A [Model] contain the same data as a [Scene], it's just stored in flat arrays instead of in a tree structure.
/// You can convert from a [Scene] to a [Model], but not the other way, because the tree structure is lost in the conversion.
///
#[derive(Debug, Clone)]
pub struct Model {
    /// The name. Might not be anything meaningful.
    pub name: String,
    /// A list of geometries for this model.
    pub geometries: Vec<Primitive>,
    /// A list of materials for this model
    pub materials: Vec<PbrMaterial>,
}

///
/// A part of a [Model] containing exactly one [Geometry], an optional reference to a material and information necessary to calculate the transformation that
/// should be applied to the geometry.
///
#[derive(Debug, Clone)]
pub struct Primitive {
    /// The name. Might not be anything meaningful.
    pub name: String,
    /// A transformation that should be applied to the [Primitive::geometry].
    pub transformation: Mat4,
    /// The geometry of this primitive.
    pub geometry: Geometry,
    /// Optional index into [Model::materials], indicating which material should be applied to [Primitive::geometry].
    pub material_index: Option<usize>,
}

impl std::ops::Deref for Primitive {
    type Target = Geometry;
    fn deref(&self) -> &Self::Target {
        &self.geometry
    }
}

impl std::ops::DerefMut for Primitive {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.geometry
    }
}

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
    #[cfg(feature = "image")]
    #[error("error while parsing an image file")]
    Image(#[from] image::ImageError),
    #[cfg(feature = "obj")]
    #[error("error while parsing an .obj file")]
    Obj(#[from] wavefront_obj::ParseError),

    #[cfg(feature = "pcd")]
    #[error("error while parsing an .pcd file")]
    Pcd(#[from] pcd_rs::anyhow::Error),

    #[cfg(not(target_arch = "wasm32"))]
    #[error("io error")]
    IO(#[from] std::io::Error),
    #[cfg(feature = "gltf")]
    #[error("error while parsing a .gltf file")]
    Gltf(#[from] ::gltf::Error),
    #[cfg(feature = "gltf")]
    #[error("the .gltf file contain corrupt buffer data")]
    GltfCorruptData,
    #[cfg(feature = "gltf")]
    #[error("the .gltf file contain missing buffer data")]
    GltfMissingData,
    #[error("the .vol file contain wrong data size")]
    VolCorruptData,
    #[cfg(not(target_arch = "wasm32"))]
    #[error("error while loading the file {0}: {1}")]
    FailedLoading(String, std::io::Error),
    #[cfg(feature = "reqwest")]
    #[error("error while loading the url {0}: {1}")]
    FailedLoadingUrl(String, reqwest::Error),
    #[cfg(feature = "reqwest")]
    #[error("error while parsing the url {0}")]
    FailedParsingUrl(String),
    #[cfg(feature = "data-url")]
    #[error("error while parsing data-url {0}: {1}")]
    FailedParsingDataUrl(String, String),
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
