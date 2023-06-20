use std::default::Default;

use bevy_ecs::{bundle::Bundle, change_detection::ResMut};

use math::Vec3;

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
        meshes: &mut ResMut<Assets<CpuMesh>>,
        materials: &mut ResMut<Assets<CpuMaterial>>,
        x: f32,
        y: f32,
        radius: f32,
        subdivisions: u32,
        color: Color,
    ) -> Self {
        Self {
            mesh: meshes.add(shapes::Circle::new(subdivisions)),
            material: materials.add(color),
            transform: Transform::from_xy(x, y).with_scale(Vec3::splat(radius)),
            ..Default::default()
        }
    }
}
