//!
//! A collection of geometries implementing the [Geometry] trait.
//!
//! A geometry together with a [material] can be rendered directly, or combined into an [object] (see [Gm]) that can be used in a render call, for example [RenderTarget::render].
//!

mod bounding_box;
mod geometry;
mod instanced_mesh;
mod mesh;

pub use bounding_box::*;
pub use geometry::*;
pub use instanced_mesh::*;
pub use mesh::*;
