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
//
// impl<T: PostMaterial + ?Sized> PostMaterial for &T {
//     fn fragment_shader(
//         &self,
//         lights: &[&dyn Light],
//         color_texture: Option<ColorTexture>,
//         depth_texture: Option<DepthTexture>,
//     ) -> FragmentShader {
//         (*self).fragment_shader(lights, color_texture, depth_texture)
//     }
//     fn use_uniforms(
//         &self,
//         program: &Program,
//         camera: &Camera,
//         lights: &[&dyn Light],
//         color_texture: Option<ColorTexture>,
//         depth_texture: Option<DepthTexture>,
//     ) {
//         (*self).use_uniforms(program, camera, lights, color_texture, depth_texture)
//     }
//     fn render_states(&self) -> RenderStates {
//         (*self).render_states()
//     }
//
//     fn material_type(&self) -> MaterialType {
//         (*self).material_type()
//     }
// }
//
// impl<T: PostMaterial + ?Sized> PostMaterial for &mut T {
//     fn fragment_shader(
//         &self,
//         lights: &[&dyn Light],
//         color_texture: Option<ColorTexture>,
//         depth_texture: Option<DepthTexture>,
//     ) -> FragmentShader {
//         (**self).fragment_shader(lights, color_texture, depth_texture)
//     }
//     fn use_uniforms(
//         &self,
//         program: &Program,
//         camera: &Camera,
//         lights: &[&dyn Light],
//         color_texture: Option<ColorTexture>,
//         depth_texture: Option<DepthTexture>,
//     ) {
//         (**self).use_uniforms(program, camera, lights, color_texture, depth_texture)
//     }
//     fn render_states(&self) -> RenderStates {
//         (**self).render_states()
//     }
//
//     fn material_type(&self) -> MaterialType {
//         (**self).material_type()
//     }
// }
//
// impl<T: PostMaterial> PostMaterial for Box<T> {
//     fn fragment_shader(
//         &self,
//         lights: &[&dyn Light],
//         color_texture: Option<ColorTexture>,
//         depth_texture: Option<DepthTexture>,
//     ) -> FragmentShader {
//         self.as_ref()
//             .fragment_shader(lights, color_texture, depth_texture)
//     }
//     fn use_uniforms(
//         &self,
//         program: &Program,
//         camera: &Camera,
//         lights: &[&dyn Light],
//         color_texture: Option<ColorTexture>,
//         depth_texture: Option<DepthTexture>,
//     ) {
//         self.as_ref()
//             .use_uniforms(program, camera, lights, color_texture, depth_texture)
//     }
//     fn render_states(&self) -> RenderStates {
//         self.as_ref().render_states()
//     }
//
//     fn material_type(&self) -> MaterialType {
//         self.as_ref().material_type()
//     }
// }
//
// impl<T: PostMaterial> PostMaterial for std::rc::Rc<T> {
//     fn fragment_shader(
//         &self,
//         lights: &[&dyn Light],
//         color_texture: Option<ColorTexture>,
//         depth_texture: Option<DepthTexture>,
//     ) -> FragmentShader {
//         self.as_ref()
//             .fragment_shader(lights, color_texture, depth_texture)
//     }
//     fn use_uniforms(
//         &self,
//         program: &Program,
//         camera: &Camera,
//         lights: &[&dyn Light],
//         color_texture: Option<ColorTexture>,
//         depth_texture: Option<DepthTexture>,
//     ) {
//         self.as_ref()
//             .use_uniforms(program, camera, lights, color_texture, depth_texture)
//     }
//     fn render_states(&self) -> RenderStates {
//         self.as_ref().render_states()
//     }
//
//     fn material_type(&self) -> MaterialType {
//         self.as_ref().material_type()
//     }
// }
//
// impl<T: PostMaterial> PostMaterial for std::sync::Arc<T> {
//     fn fragment_shader(
//         &self,
//         lights: &[&dyn Light],
//         color_texture: Option<ColorTexture>,
//         depth_texture: Option<DepthTexture>,
//     ) -> FragmentShader {
//         self.as_ref()
//             .fragment_shader(lights, color_texture, depth_texture)
//     }
//     fn use_uniforms(
//         &self,
//         program: &Program,
//         camera: &Camera,
//         lights: &[&dyn Light],
//         color_texture: Option<ColorTexture>,
//         depth_texture: Option<DepthTexture>,
//     ) {
//         self.as_ref()
//             .use_uniforms(program, camera, lights, color_texture, depth_texture)
//     }
//     fn render_states(&self) -> RenderStates {
//         self.as_ref().render_states()
//     }
//
//     fn material_type(&self) -> MaterialType {
//         self.as_ref().material_type()
//     }
// }
//
// impl<T: PostMaterial> PostMaterial for std::cell::RefCell<T> {
//     fn fragment_shader(
//         &self,
//         lights: &[&dyn Light],
//         color_texture: Option<ColorTexture>,
//         depth_texture: Option<DepthTexture>,
//     ) -> FragmentShader {
//         self.borrow()
//             .fragment_shader(lights, color_texture, depth_texture)
//     }
//     fn use_uniforms(
//         &self,
//         program: &Program,
//         camera: &Camera,
//         lights: &[&dyn Light],
//         color_texture: Option<ColorTexture>,
//         depth_texture: Option<DepthTexture>,
//     ) {
//         self.borrow()
//             .use_uniforms(program, camera, lights, color_texture, depth_texture)
//     }
//     fn render_states(&self) -> RenderStates {
//         self.borrow().render_states()
//     }
//
//     fn material_type(&self) -> MaterialType {
//         self.borrow().material_type()
//     }
// }
//
// impl<T: PostMaterial> PostMaterial for std::sync::RwLock<T> {
//     fn fragment_shader(
//         &self,
//         lights: &[&dyn Light],
//         color_texture: Option<ColorTexture>,
//         depth_texture: Option<DepthTexture>,
//     ) -> FragmentShader {
//         self.read()
//             .unwrap()
//             .fragment_shader(lights, color_texture, depth_texture)
//     }
//     fn use_uniforms(
//         &self,
//         program: &Program,
//         camera: &Camera,
//         lights: &[&dyn Light],
//         color_texture: Option<ColorTexture>,
//         depth_texture: Option<DepthTexture>,
//     ) {
//         self.read()
//             .unwrap()
//             .use_uniforms(program, camera, lights, color_texture, depth_texture)
//     }
//     fn render_states(&self) -> RenderStates {
//         self.read().unwrap().render_states()
//     }
//
//     fn material_type(&self) -> MaterialType {
//         self.read().unwrap().material_type()
//     }
// }
