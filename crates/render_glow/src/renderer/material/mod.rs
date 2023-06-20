//!
//! A collection of materials implementing the [Material] and/or [PostMaterial] trait.
//!
//! A material together with a [geometry] can be rendered directly (using [Geometry::render_with_material] or [Geometry::render_with_post_material]).
//! A [Material] can also be combined into an [object] (see [Gm]) and be used in a render call, for example [RenderTarget::render].
//!

pub use color_material::*;
pub use color_texture::*;
pub use cpu_pbr_utils::*;
pub use depth_material::*;
pub use depth_texture::*;
pub use fragment_attributes::*;
pub use fragment_shader::*;
pub use material::*;
pub use orm_material::*;
pub use physical_material::*;
pub use position_material::*;
pub use texture_2d_ref::*;
pub use uv_material::*;

mod color_material;
mod color_texture;
mod cpu_pbr_utils;
mod depth_material;
mod depth_texture;
mod fragment_attributes;
mod fragment_shader;
mod material;
mod orm_material;
mod physical_material;
mod position_material;
mod texture_2d_ref;
mod uv_material;
