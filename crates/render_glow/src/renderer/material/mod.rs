//!
//! A collection of materials implementing the [Material] and/or [PostMaterial] trait.
//!
//! A material together with a [geometry] can be rendered directly (using [Geometry::render_with_material] or [Geometry::render_with_post_material]).
//! A [Material] can also be combined into an [object] (see [Gm]) and be used in a render call, for example [RenderTarget::render].
//!

pub use cpu_pbr_utils::*;
pub use depth_material::*;
pub use fragment_attributes::*;
pub use fragment_shader::*;
pub use material::*;
pub use pbr_material::*;

mod cpu_pbr_utils;
mod depth_material;
mod fragment_attributes;
mod fragment_shader;
mod material;
mod pbr_material;
