// use math::*;
//
// use render_api::base::{AxisAlignedBoundingBox, Camera, TriMesh};
//
// use crate::{
//     core::{ColorTexture, DepthTexture},
//     renderer::*,
// };
//
// ///
// /// A bounding box geometry used for visualising an [AxisAlignedBoundingBox].
// ///
// pub struct BoundingBox {
//     mesh: InstancedMesh,
//     aabb: AxisAlignedBoundingBox,
// }
//
// impl BoundingBox {
//     ///
//     /// Creates a bounding box geometry from an axis aligned bounding box.
//     ///
//     pub fn new(aabb: AxisAlignedBoundingBox) -> Self {
//         let size = aabb.size();
//         let thickness = 0.02 * size.x.max(size.y).max(size.z);
//
//         Self::new_with_thickness(aabb, thickness)
//     }
//
//     ///
//     /// Creates a bounding box object from an axis aligned bounding box with a specified line
//     /// thickness.
//     ///
//     pub fn new_with_thickness(aabb: AxisAlignedBoundingBox, thickness: f32) -> Self {
//         let max = aabb.max();
//         let min = aabb.min();
//         let size = aabb.size();
//         let translations = vec![
//             min,
//             Vec3::new(min.x, max.y, max.z),
//             Vec3::new(min.x, min.y, max.z),
//             Vec3::new(min.x, max.y, min.z),
//             min,
//             Vec3::new(max.x, min.y, max.z),
//             Vec3::new(min.x, min.y, max.z),
//             Vec3::new(max.x, min.y, min.z),
//             min,
//             Vec3::new(max.x, max.y, min.z),
//             Vec3::new(min.x, max.y, min.z),
//             Vec3::new(max.x, min.y, min.z),
//         ];
//
//         let mesh = InstancedMesh::new(
//             &Instances {
//                 transformations: (0..12)
//                     .map(|i| {
//                         Mat4::from_translation(translations[i])
//                             * match i {
//                                 0..=3 => Mat4::from_scale(Vec3::new(size.x, thickness, thickness)),
//                                 4..=7 => {
//                                     Mat4::from_rotation_z(f32::to_radians(90.0))
//                                         * Mat4::from_scale(Vec3::new(size.y, thickness, thickness))
//                                 }
//                                 8..=11 => {
//                                     Mat4::from_rotation_y(f32::to_radians(-90.0))
//                                         * Mat4::from_scale(Vec3::new(size.z, thickness, thickness))
//                                 }
//                                 _ => unreachable!(),
//                             }
//                     })
//                     .collect(),
//                 ..Default::default()
//             },
//             &TriMesh::cylinder(16),
//         );
//         Self { mesh, aabb }
//     }
// }
//
// impl<'a> IntoIterator for &'a BoundingBox {
//     type Item = &'a dyn Geometry;
//     type IntoIter = std::iter::Once<&'a dyn Geometry>;
//
//     fn into_iter(self) -> Self::IntoIter {
//         std::iter::once(self)
//     }
// }
//
// impl Geometry for BoundingBox {
//     fn aabb(&self) -> AxisAlignedBoundingBox {
//         self.aabb
//     }
//
//     fn render_with_material(
//         &self,
//         material: &dyn Material,
//         camera: &Camera,
//         lights: &[&dyn Light],
//     ) {
//         self.mesh.render_with_material(material, camera, lights)
//     }
//
//     fn render_with_post_material(
//         &self,
//         material: &dyn PostMaterial,
//         camera: &Camera,
//         lights: &[&dyn Light],
//         color_texture: Option<ColorTexture>,
//         depth_texture: Option<DepthTexture>,
//     ) {
//         self.mesh
//             .render_with_post_material(material, camera, lights, color_texture, depth_texture)
//     }
// }
