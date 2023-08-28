use std::default::Default;

use bevy_ecs::bundle::Bundle;
use bevy_log::info;

use math::{Vec2, Vec3};

use crate::{
    assets::Handle,
    base::{Indices, Positions, Color, CpuMaterial, CpuMesh},
    components::Visibility,
    shapes,
    shapes::set_2d_line_transform,
    Assets,
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
        position: Vec2,
        radius: f32,
        subdivisions: u16,
        color: Color,
        outline: Option<u8>,
    ) -> Self {
        let mesh = if let Some(_thickness) = outline {
            let mesh = meshes.add(shapes::HollowCircle::new(subdivisions));
            mesh
        } else {
            let mesh = meshes.add(shapes::Circle::new(subdivisions));
            mesh
        };

        Self {
            mesh,
            material: materials.add(color),
            transform: Transform::from_translation_2d(position).with_scale(Vec3::splat(radius)),
            ..Default::default()
        }
    }

    pub fn square(
        meshes: &mut Assets<CpuMesh>,
        materials: &mut Assets<CpuMaterial>,
        position: Vec2,
        size: f32,
        color: Color,
        outline_only: bool,
    ) -> Self {
        Self::rectangle(
            meshes,
            materials,
            position,
            Vec2::splat(size),
            color,
            outline_only,
        )
    }

    pub fn rectangle(
        meshes: &mut Assets<CpuMesh>,
        materials: &mut Assets<CpuMaterial>,
        position: Vec2,
        size: Vec2,
        color: Color,
        outline_only: bool,
    ) -> Self {
        if outline_only {
            let mesh = meshes.add(shapes::HollowRectangle::new(
                (size.x * 1000.0) as u32,
                (size.y * 1000.0) as u32,
            ));
            Self {
                mesh,
                material: materials.add(color),
                transform: Transform::from_translation_2d(position),
                ..Default::default()
            }
        } else {
            let mesh = meshes.add(shapes::Square::new());
            Self {
                mesh,
                material: materials.add(color),
                transform: Transform::from_translation_2d(position).with_scale(size.extend(0.0)),
                ..Default::default()
            }
        }
    }

    pub fn line(
        meshes: &mut Assets<CpuMesh>,
        materials: &mut Assets<CpuMaterial>,
        start: Vec2,
        end: Vec2,
        color: Color,
    ) -> Self {
        let mesh = meshes.add(shapes::Line);
        let mut transform = Transform::default();
        set_2d_line_transform(&mut transform, start, end);
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
        position: Vec3,
        radius: f32,
        subdivisions: u16,
        color: Color,
    ) -> Self {
        let mesh = meshes.add(shapes::Sphere::new(subdivisions));
        Self {
            mesh,
            material: materials.add(color),
            transform: Transform::from_translation(position).with_scale(Vec3::splat(radius)),
            ..Default::default()
        }
    }

    pub fn cube(
        meshes: &mut Assets<CpuMesh>,
        materials: &mut Assets<CpuMaterial>,
        position: Vec3,
        size: f32,
        color: Color,
    ) -> Self {
        let mesh = meshes.add(shapes::Cube);
        Self {
            mesh,
            material: materials.add(color),
            transform: Transform::from_translation(position).with_scale(Vec3::splat(size)),
            ..Default::default()
        }
    }

    pub fn world_triangle(
        meshes: &mut Assets<CpuMesh>,
        materials: &mut Assets<CpuMaterial>,
        positions: [Vec3; 3],
        color: Color,
    ) -> Self {

        let mut outer_a = positions[0];
        let mut outer_b = positions[1];
        let mut outer_c = positions[2];

        let center = Vec3::new(
            (outer_a.x + outer_b.x + outer_c.x) / 3.0,
            (outer_a.y + outer_b.y + outer_c.y) / 3.0,
            (outer_a.z + outer_b.z + outer_c.z) / 3.0,
        );

        outer_a -= center;
        outer_b -= center;
        outer_c -= center;

        let positions = vec![outer_a, outer_b, outer_c];
        let indices: Indices = Indices(Some(vec![0u16, 1, 2]));

        let mut mesh = CpuMesh {
            indices,
            positions: Positions(positions),
            ..Default::default()
        };

        mesh.compute_normals();

        Self {
            mesh: meshes.add_unique(mesh),
            material: materials.add(color),
            transform: Transform::from_translation(center),
            ..Default::default()
        }
    }

    pub fn equilateral_triangle(
        meshes: &mut Assets<CpuMesh>,
        materials: &mut Assets<CpuMaterial>,
        position: Vec2,
        size: f32,
        color: Color,
        outline: Option<u8>,
    ) -> Self {
        let mesh = if let Some(_thickness) = outline {
            let mesh = meshes.add(shapes::HollowTriangle::new_2d_equilateral());
            mesh
        } else {
            let mesh = meshes.add(shapes::Triangle::new_2d_equilateral());
            mesh
        };

        Self {
            mesh,
            material: materials.add(color),
            transform: Transform::from_translation_2d(position).with_scale(Vec3::splat(size)),
            ..Default::default()
        }
    }
}
