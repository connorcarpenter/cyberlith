use bevy_ecs::{entity::Entity, prelude::Component};
use math::Vec3;
use render_api::Assets;
use render_api::base::{Color, CpuMaterial, CpuMesh, Indices, Positions};
use render_api::components::{RenderObjectBundle, Transform};

#[derive(Component)]
pub struct Edge2d {
    pub start: Entity,
    pub end_3d: Entity,
}

impl Edge2d {
    pub fn new(start: Entity, end: Entity) -> Self {
        Self { start, end_3d: end }
    }
}

#[derive(Component)]
pub struct Edge3d {
    pub start: Entity,
    pub end: Entity,
}

impl Edge3d {
    pub fn new(start: Entity, end: Entity) -> Self {
        Self { start, end }
    }
}

pub fn create_3d_edge_diamond(
    meshes: &mut Assets<CpuMesh>,
    materials: &mut Assets<CpuMaterial>,
    start: Vec3,
    end: Vec3,
    color: Color,
) -> RenderObjectBundle {
    let mesh = meshes.add(create_3d_edge_diamond_mesh());
    let distance = start.distance(end);
    let mut transform = Transform::from_translation(start)
        .looking_at(end, Vec3::Y)
        .with_scale(Vec3::new(distance, 1.0, 1.0));
    RenderObjectBundle {
        mesh,
        material: materials.add(color),
        transform,
        ..Default::default()
    }
}

fn create_3d_edge_diamond_mesh() -> CpuMesh {

    let positions = vec![
        Vec3::new(0.0, 0.0, 0.0),

        Vec3::new(0.8,  1.0,  0.0),
        Vec3::new(0.8, -0.5,  0.866),
        Vec3::new(0.8, -0.5, -0.866),

        Vec3::new(1.0, 0.0, 0.0),
    ];

    let indices: Indices = Indices(Some(vec![
        0u16, 1, 2,
        0, 1, 3,
        0, 2, 3,
        4, 1, 2,
        4, 1, 3,
        4, 2, 3,
    ]));

    let mut mesh = CpuMesh {
        indices,
        positions: Positions(positions),
        ..Default::default()
    };

    mesh.compute_normals();

    mesh
}