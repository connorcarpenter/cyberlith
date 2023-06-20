//!
//! A collection of geometries implementing the [Geometry] trait.
//!
//! A geometry together with a [material] can be rendered directly, or combined into an [object] (see [Gm]) that can be used in a render call, for example [RenderTarget::render].
//!

pub use base_mesh::*;
pub use geometry::*;
pub use instanced_mesh::*;
pub use mesh::*;

mod base_mesh;
mod geometry;
mod instanced_mesh;
mod mesh;
