use std::default::Default;

use bevy_ecs::component::Component;

use crate::base::Vec3;

#[derive(Clone, Component, Default, Copy)]
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
