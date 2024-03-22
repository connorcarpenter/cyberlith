pub use glam::{Affine3A, EulerRot, Mat2, Mat3, Mat4, Quat, Vec2, Vec3, Vec4};

mod conversion;
mod quat;
mod winding;

pub use conversion::*;
pub use quat::*;
pub use winding::*;

////

use rand::{thread_rng, Rng};

pub fn generate_random_range_f32(min: f32, max: f32) -> f32 {
    thread_rng().gen_range(min..max)
}