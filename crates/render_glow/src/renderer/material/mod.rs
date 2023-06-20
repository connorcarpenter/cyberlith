//!
//! A collection of materials implementing the [Material] and/or [PostMaterial] trait.
//!
//! A material together with a [geometry] can be rendered directly (using [Geometry::render_with_material] or [Geometry::render_with_post_material]).
//! A [Material] can also be combined into an [object] (see [Gm]) and be used in a render call, for example [RenderTarget::render].
//!

pub use color_material::*;
pub use cpu_pbr_utils::*;
pub use depth_material::*;
pub use fragment_attributes::*;
pub use fragment_shader::*;
pub use material::*;
pub use orm_material::*;
pub use pbr_material::*;
pub use position_material::*;
pub use uv_material::*;

mod color_material;
mod cpu_pbr_utils;
mod depth_material;
mod fragment_attributes;
mod fragment_shader;
mod material;
mod orm_material;
mod pbr_material;
mod position_material;
mod uv_material;
