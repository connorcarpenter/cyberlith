use std::default::Default;

use bevy_ecs::{bundle::Bundle, change_detection::ResMut};

use math::{Vec2, Vec3};

use crate::{
    Assets,
    assets::Handle,
    base::{Color, CpuMaterial, CpuMesh}, shapes,
};

use super::transform::Transform;

#[derive(Default, Bundle)]
pub struct RenderObjectBundle {
    pub mesh: Handle<CpuMesh>,
    pub material: Handle<CpuMaterial>,
    pub transform: Transform,
}

impl RenderObjectBundle {
    pub fn circle(
        meshes: &mut Assets<CpuMesh>,
        materials: &mut Assets<CpuMaterial>,
        x: f32,
        y: f32,
        radius: f32,
        subdivisions: u16,
        color: Color,
        outline_only: bool,
    ) -> Self {
        if outline_only {
            let mesh = meshes.add(shapes::HollowCircle::new(
                subdivisions,
                (radius * 1000.0) as u32,
            ));
            Self {
                mesh,
                material: materials.add(color),
                transform: Transform::from_xy(x, y),
                ..Default::default()
            }
        } else {
            let mesh = meshes.add(shapes::Circle::new(subdivisions));
            Self {
                mesh,
                material: materials.add(color),
                transform: Transform::from_xy(x, y).with_scale(Vec3::splat(radius)),
                ..Default::default()
            }
        }
    }

    pub fn square(
        meshes: &mut ResMut<Assets<CpuMesh>>,
        materials: &mut ResMut<Assets<CpuMaterial>>,
        x: f32,
        y: f32,
        size: f32,
        color: Color,
        outline_only: bool,
    ) -> Self {
        Self::rectangle(meshes, materials, x, y, size, size, color, outline_only)
    }

    pub fn rectangle(
        meshes: &mut ResMut<Assets<CpuMesh>>,
        materials: &mut ResMut<Assets<CpuMaterial>>,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        color: Color,
        outline_only: bool,
    ) -> Self {
        if outline_only {
            let mesh = meshes.add(shapes::HollowRectangle::new(
                (width * 1000.0) as u32,
                (height * 1000.0) as u32,
            ));
            Self {
                mesh,
                material: materials.add(color),
                transform: Transform::from_xy(x, y),
                ..Default::default()
            }
        } else {
            let mesh = meshes.add(shapes::Square::new());
            Self {
                mesh,
                material: materials.add(color),
                transform: Transform::from_xy(x, y)
                    .with_scale(Vec2::new(width, height).extend(0.0)),
                ..Default::default()
            }
        }
    }

    pub fn sphere(
        meshes: &mut Assets<CpuMesh>,
        materials: &mut Assets<CpuMaterial>,
        x: f32,
        y: f32,
        z: f32,
        radius: f32,
        subdivisions: u16,
        color: Color,
    ) -> Self {
        let mesh = meshes.add(shapes::Sphere::new(subdivisions));
        Self {
            mesh,
            material: materials.add(color),
            transform: Transform::from_xyz(x, y, z).with_scale(Vec3::splat(radius)),
            ..Default::default()
        }
    }
}
