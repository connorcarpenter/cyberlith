use std::default::Default;

use bevy_ecs::{bundle::Bundle, component::Component};

use crate::Transform;

#[derive(Bundle, Default)]
pub struct PointLightBundle {
    pub point_light: PointLight,
    pub transform: Transform,
}

#[derive(Component, Default)]
pub struct PointLight {
    pub intensity: f32,
}