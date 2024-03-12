use crate::components::{AmbientLight, DirectionalLight, PointLight};

pub enum TypedLight {
    Ambient(AmbientLight),
    Directional(DirectionalLight),
    Point(PointLight),
}
