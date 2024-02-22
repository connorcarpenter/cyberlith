//!
//! A collection of lights implementing the [Light] trait.
//!
//! Lights shines onto objects in the scene, note however that some materials are affected by lights, others are not.
//!

mod ambient_light;
mod directional_light;
mod light;
mod point_light;
mod utils;

pub use light::*;
pub use utils::*;
