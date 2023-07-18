use std::default::Default;

use bevy_ecs::bundle::Bundle;
use bevy_log::info;

use math::{Vec2, Vec3};

use crate::{
    Assets,
    assets::Handle,
    base::{Color, CpuMaterial, CpuMesh}, components::Visibility,
    shapes, shapes::set_line_transform,
};

use super::transform::Transform;

#[derive(Default, Bundle)]
pub struct RenderObjectBundle {
    pub mesh: Handle<CpuMesh>,
    pub material: Handle<CpuMaterial>,
    pub transform: Transform,
    pub visibility: Visibility,
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
        outline: Option<u8>,
    ) -> Self {
        let mesh = if let Some(_thickness) = outline {
            let mesh = meshes.add(shapes::HollowCircle::new(
                subdivisions
            ));
            info!("hollow mesh: {:?}", mesh.id);
            mesh
        } else {
            let mesh = meshes.add(shapes::Circle::new(subdivisions));
            info!("solid mesh: {:?}", mesh.id);
            mesh
        };

        Self {
            mesh,
            material: materials.add(color),
            transform: Transform::from_xy(x, y).with_scale(Vec3::splat(radius)),
            ..Default::default()
        }
    }

    pub fn square(
        meshes: &mut Assets<CpuMesh>,
        materials: &mut Assets<CpuMaterial>,
        x: f32,
        y: f32,
        size: f32,
        color: Color,
        outline_only: bool,
    ) -> Self {
        Self::rectangle(meshes, materials, x, y, size, size, color, outline_only)
    }

    pub fn rectangle(
        meshes: &mut Assets<CpuMesh>,
        materials: &mut Assets<CpuMaterial>,
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

    pub fn line(
        meshes: &mut Assets<CpuMesh>,
        materials: &mut Assets<CpuMaterial>,
        start: &Vec2,
        end: &Vec2,
        color: Color,
    ) -> Self {
        let mesh = meshes.add(shapes::Line::new());
        let mut transform = Transform::default();
        set_line_transform(&mut transform, start, end);
        Self {
            mesh,
            material: materials.add(color),
            transform,
            ..Default::default()
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

    pub fn cube(
        meshes: &mut Assets<CpuMesh>,
        materials: &mut Assets<CpuMaterial>,
        x: f32,
        y: f32,
        z: f32,
        size: f32,
        color: Color,
    ) -> Self {
        let mesh = meshes.add(shapes::Cube);
        Self {
            mesh,
            material: materials.add(color),
            transform: Transform::from_xyz(x, y, z).with_scale(Vec3::splat(size)),
            ..Default::default()
        }
    }
}
