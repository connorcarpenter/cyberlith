use std::default::Default;

use crate::math::Vec3;
use bevy_ecs::component::Component;

#[derive(Component, Default)]
pub struct Transform {}

impl Transform {
    pub fn looking_at(mut self, target: Vec3, up: Vec3) -> Self {
        self
    }
}

impl Transform {
    pub fn from_xyz(x: f32, y: f32, z: f32) -> Self {
        Self {}
    }
}
