use std::default::Default;

use bevy_ecs::component::Component;

use math::{matrix_transform_point, Affine3A, Mat3, Mat4, Quat, Vec2, Vec3};

#[derive(Clone, Component, Copy, Debug)]
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

    pub fn from_xy(x: f32, y: f32) -> Self {
        Self::from_translation(Vec3::new(x, y, 0.0))
    }

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

    pub fn from_translation_2d(translation: Vec2) -> Self {
        Transform {
            translation: translation.extend(0.0),
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

    pub fn compute_affine(&self) -> Affine3A {
        Affine3A::from_scale_rotation_translation(self.scale, self.rotation, self.translation)
    }

    pub fn orbit_rotate(&mut self, delta: Vec2) {
        self.rotate_y(delta.y);
        self.rotate_x(delta.x);
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

    /// Rotates this [`Transform`] so that [`Transform::forward`] points in the given `direction`
    /// and [`Transform::up`] points towards `up`.
    ///
    /// * if `direction` is parallel with `up`, an orthogonal vector is used as the "right" direction
    pub fn look_to(&mut self, direction: Vec3, up: Vec3) {
        // bevy impl:
        // let back = -direction.try_normalize().unwrap();
        // let up = up.try_normalize().unwrap();
        // let right = up
        //     .cross(back)
        //     .try_normalize()
        //     .unwrap_or_else(|| up.any_orthonormal_vector());
        // let up = back.cross(right);
        // self.rotation = Quat::from_mat3(&Mat3::from_cols(right, up, back));

        let Some(forward) = direction.try_normalize() else {
            panic!("invalid direction: {:?}", direction);
        };
        let up = up.try_normalize().unwrap();
        let right = up.cross(forward).try_normalize().unwrap();
        let up = forward.cross(right).normalize();

        self.rotation = Quat::from_mat3(&Mat3::from_cols(right, up, forward));
    }

    pub fn set_scale(&mut self, scale: Vec3) {
        self.scale = scale;
    }

    pub fn set_translation(&mut self, translation: Vec3) {
        self.translation = translation;
    }

    pub fn set_rotation(&mut self, rotation: Quat) {
        self.rotation = rotation;
    }

    pub fn view_matrix(&self) -> Mat4 {
        self.compute_matrix().inverse()
    }

    pub fn inverse(&self) -> Self {
        let inverse_matrix = self.compute_matrix().inverse();
        Transform::from_matrix(inverse_matrix)
    }

    pub fn multiply(&self, transform: &Transform) -> Self {
        let translation = transform.transform_point(&self.translation);
        let rotation = transform.rotation * self.rotation;
        let scale = transform.scale * self.scale;
        Transform {
            translation,
            rotation,
            scale,
        }
    }

    pub fn transform_point(&self, point: &Vec3) -> Vec3 {
        let matrix = self.compute_matrix();
        matrix_transform_point(&matrix, point)
    }

    pub fn local_x(&self) -> Vec3 {
        self.rotation * Vec3::X
    }

    pub fn local_y(&self) -> Vec3 {
        self.rotation * Vec3::Y
    }

    pub fn local_z(&self) -> Vec3 {
        self.rotation * Vec3::Z
    }

    pub fn view_right(&self) -> Vec3 {
        self.right().cross(self.up())
    }

    pub fn view_down(&self) -> Vec3 {
        self.view_right().cross(self.up())
    }

    pub fn view_forward(&self) -> Vec3 {
        self.left().cross(self.forward())
    }

    pub fn right(&self) -> Vec3 {
        self.local_y()
    }

    pub fn left(&self) -> Vec3 {
        -self.local_y()
    }

    pub fn up(&self) -> Vec3 {
        self.local_z()
    }

    pub fn down(&self) -> Vec3 {
        -self.local_z()
    }

    pub fn forward(&self) -> Vec3 {
        self.local_x()
    }

    pub fn back(&self) -> Vec3 {
        -self.local_x()
    }

    pub fn mirror(&mut self, other: &Self) {
        self.translation = other.translation;
        self.rotation = other.rotation;
        self.scale = other.scale;
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl From<Affine3A> for Transform {
    fn from(value: Affine3A) -> Self {
        let (scale, rotation, translation) = value.to_scale_rotation_translation();
        Transform::from_translation(translation)
            .with_scale(scale)
            .with_rotation(rotation)
    }
}
