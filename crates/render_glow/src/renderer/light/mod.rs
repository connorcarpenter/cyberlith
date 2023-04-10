//!
//! A collection of lights implementing the [Light] trait.
//!
//! Lights shines onto objects in the scene, note however that some materials are affected by lights, others are not.
//!

mod ambient_light;
mod directional_light;
mod environment;
mod light;
mod point_light;
mod utils;

pub use ambient_light::*;
pub use directional_light::*;
pub use environment::*;
pub use light::*;
pub use point_light::*;
pub use utils::*;
