use std::{default::Default, ops::Mul};

use bevy_ecs::component::Component;

use math::{Mat3, Mat4, Quat, Vec3};

#[derive(Clone, Component, Copy)]
pub struct Transform {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Transform {

    pub const IDENTITY: Self = Transform {
        translation: Vec3::ZERO,
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
    };
    
    pub fn from_xyz(x: f32, y: f32, z: f32) -> Self {
        Self::from_translation(Vec3::new(x, y, z))
    }

    pub fn from_matrix(matrix: Mat4) -> Self {
        let (scale, rotation, translation) = matrix.to_scale_rotation_translation();

        Transform {
            translation,
            rotation,
            scale,
        }
    }

    pub fn from_translation(translation: Vec3) -> Self {
        Transform {
            translation,
            ..Self::IDENTITY
        }
    }

    pub fn from_rotation(rotation: Quat) -> Self {
        Transform {
            rotation,
            ..Self::IDENTITY
        }
    }

    pub const fn from_scale(scale: Vec3) -> Self {
        Transform {
            scale,
            ..Self::IDENTITY
        }
    }

    pub fn from_axis_angle(axis: Vec3, angle_radians: f32) -> Transform {
        let rotation = Quat::from_axis_angle(axis, angle_radians);
        Transform::from_rotation(rotation)
    }

    pub const fn with_translation(mut self, translation: Vec3) -> Self {
        self.translation = translation;
        self
    }

    pub const fn with_rotation(mut self, rotation: Quat) -> Self {
        self.rotation = rotation;
        self
    }

    pub const fn with_scale(mut self, scale: Vec3) -> Self {
        self.scale = scale;
        self
    }

    pub fn compute_matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.translation)
    }

    pub fn rotate(&mut self, rotation: Quat) {
        self.rotation = rotation * self.rotation;
    }

    pub fn rotate_axis(&mut self, axis: Vec3, angle: f32) {
        self.rotate(Quat::from_axis_angle(axis, angle));
    }

    pub fn rotate_x(&mut self, angle: f32) {
        self.rotate(Quat::from_rotation_x(angle));
    }
    
    pub fn rotate_y(&mut self, angle: f32) {
        self.rotate(Quat::from_rotation_y(angle));
    }
    
    pub fn rotate_z(&mut self, angle: f32) {
        self.rotate(Quat::from_rotation_z(angle));
    }

    pub fn looking_at(mut self, target: Vec3, up: Vec3) -> Self {
        self.look_at(target, up);
        self
    }

    pub fn looking_to(mut self, direction: Vec3, up: Vec3) -> Self {
        self.look_to(direction, up);
        self
    }

    pub fn look_at(&mut self, target: Vec3, up: Vec3) {
        self.look_to(target - self.translation, up);
    }
    
    pub fn look_to(&mut self, direction: Vec3, up: Vec3) {
        let forward = -direction.normalize();
        let right = up.cross(forward).normalize();
        let up = forward.cross(right);
        self.rotation = Quat::from_mat3(&Mat3::from_cols(right, up, forward));
    }

    pub fn view_matrix(&self) -> Mat4 {

        let (position, target, up) = self.get_position_target_up();

        return Mat4::look_at_rh(
            position,
            target,
            up,
        );
    }

    pub fn get_position_target_up(&self) -> (Vec3, Vec3, Vec3) {
        let position = self.translation;
        let target = self.translation + self.rotation * Vec3::Z;
        let up = self.rotation * Vec3::Y;
        (position, target, up)
    }

    pub fn mul_transform(&self, transform: Transform) -> Self {
        let translation = self.transform_point(transform.translation);
        let rotation = self.rotation * transform.rotation;
        let scale = self.scale * transform.scale;
        Transform {
            translation,
            rotation,
            scale,
        }
    }

    pub fn transform_point(&self, mut point: Vec3) -> Vec3 {
        point = self.scale * point;
        point = self.rotation * point;
        point += self.translation;
        point
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Mul<Transform> for Transform {
    type Output = Transform;

    fn mul(self, transform: Transform) -> Self::Output {
        self.mul_transform(transform)
    }
}