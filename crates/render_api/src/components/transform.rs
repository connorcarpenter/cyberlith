use std::default::Default;

use bevy_ecs::component::Component;
use cgmath::{InnerSpace, Rotation3};

use crate::base::{Mat3, Mat4, Quat, Vec3};

#[derive(Clone, Component, Copy)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Quat,
}

impl Transform {
    pub fn from_xyz(x: f32, y: f32, z: f32) -> Self {
        let position = Vec3::new(x, y, z);
        let rotation = Quat::from_angle_y(cgmath::Deg(0.0));

        Self { position, rotation }
    }

    pub fn from_axis_angle(axis: Vec3, angle: f32) -> Self {
        let rotation = Quat::from_axis_angle(axis.normalize(), cgmath::Deg(angle));
        let position = Vec3::new(0.0, 0.0, 0.0);

        Self { position, rotation }
    }

    pub fn to_mat4(&self) -> Mat4 {
        // convert translation & rotation into a 4x4 matrix
        let rotation_matrix: Mat4 = self.rotation.into();
        let translation_matrix = Mat4::from_translation(self.position);
        translation_matrix * rotation_matrix
    }

    pub fn looking_at(mut self, target: Vec3, up: Vec3) -> Self {
        let forward = (target - self.position).normalize();
        let right = up.cross(forward).normalize();
        let up = forward.cross(right);

        let basis = Mat3::from_cols(right, up, forward);
        self.rotation = Quat::from(basis);

        self
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Vec3::new(0.0, 0.0, 0.0),
            rotation: Quat::from_angle_y(cgmath::Deg(0.0)),
        }
    }
}
