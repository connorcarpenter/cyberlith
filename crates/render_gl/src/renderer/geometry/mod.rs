//!
//! A collection of geometries implementing the [Geometry] trait.
//!
//! A geometry together with a [material] can be rendered directly, or combined into an [object] (see [Gm]) that can be used in a render call, for example [RenderTarget::render].
//!

pub use gpu_mesh::*;
pub use instances::*;

mod gpu_mesh;
mod instances;
