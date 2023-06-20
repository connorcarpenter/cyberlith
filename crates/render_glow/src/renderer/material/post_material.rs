// use render_api::components::Camera;
//
// use crate::{
//     core::{ColorTexture, DepthTexture, Program, RenderStates},
//     renderer::{FragmentShader, Light, MaterialType},
// };
//
// ///
// /// Similar to [Material], the difference is that this type of material needs the rendered color texture and/or depth texture of the scene to be applied.
// /// Therefore this type of material is always applied one at a time and after the scene has been rendered with the regular [Material].
// ///
// pub trait PostMaterial {
//     ///
//     /// Returns a [FragmentShader], ie. the fragment shader source for this material
//     /// and a [FragmentAttributes] struct that describes which fragment attributes are required for rendering with this material.
//     ///
//     fn fragment_shader(
//         &self,
//         lights: &[&dyn Light],
//         color_texture: Option<ColorTexture>,
//         depth_texture: Option<DepthTexture>,
//     ) -> FragmentShader;
//
//     ///
//     /// Sends the uniform data needed for this material to the fragment shader.
//     ///
//     fn use_uniforms(
//         &self,
//         program: &Program,
//         camera: &Camera,
//         lights: &[&dyn Light],
//         color_texture: Option<ColorTexture>,
//         depth_texture: Option<DepthTexture>,
//     );
//
//     ///
//     /// Returns the render states needed to render with this material.
//     ///
//     fn render_states(&self) -> RenderStates;
//
//     ///
//     /// Returns the type of material.
//     ///
//     fn material_type(&self) -> MaterialType;
// }
