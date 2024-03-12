use std::collections::HashMap;

use math::Mat4;
use render_api::{resources::MaterialOrSkinHandle, base::CpuMesh};
use storage::Handle;

use crate::renderer::RenderCamera;

///
/// Compare function for sorting objects based on distance from the camera.
/// The order is opaque objects from nearest to farthest away from the camera,
/// then transparent objects from farthest away to closest to the camera.
///
pub fn cmp_render_order(
    _camera: &RenderCamera,
    _obj0: &HashMap<Handle<CpuMesh>, Vec<(MaterialOrSkinHandle, Mat4)>>,
    _obj1: &HashMap<Handle<CpuMesh>, Vec<(MaterialOrSkinHandle, Mat4)>>,
) -> std::cmp::Ordering {
    todo!()
    // let distance_a = camera
    //     .transform
    //     .translation
    //     .distance_squared(obj0.aabb().center());
    // let distance_b = camera
    //     .transform
    //     .translation
    //     .distance_squared(obj1.aabb().center());
    // if distance_a.is_nan() || distance_b.is_nan() {
    //     distance_a.is_nan().cmp(&distance_b.is_nan()) // whatever - just save us from panicing on unwrap below
    // } else {
    //     distance_a.partial_cmp(&distance_b).unwrap()
    // }
}
