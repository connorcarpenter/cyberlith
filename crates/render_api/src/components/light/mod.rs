mod ambient_light;
mod directional_light;
mod point_light;

pub use ambient_light::*;
pub use directional_light::*;
pub use point_light::*;

///
/// Specifies how the intensity of a light fades over distance.
/// The light intensity is scaled by ``` 1 / max(1, constant + distance * linear + distance * distance * quadratic) ```.
///
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Attenuation {
    /// Constant attenuation factor.
    pub constant: f32,
    /// Linear attenuation factor.
    pub linear: f32,
    /// Quadratic attenuation factor.
    pub quadratic: f32,
}

impl Default for Attenuation {
    fn default() -> Self {
        Self {
            constant: 1.0,
            linear: 0.0,
            quadratic: 0.0,
        }
    }
}
