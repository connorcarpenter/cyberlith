use math::Vec3;
use render_api::{
    base::{Color, CpuMaterial, CpuMesh, Indices, Positions},
    components::{RenderObjectBundle, Transform},
    AssetHash, Assets,
};

pub fn create_3d_edge_diamond(
    meshes: &mut Assets<CpuMesh>,
    materials: &mut Assets<CpuMaterial>,
    start: Vec3,
    end: Vec3,
    color: Color,
    thickness: f32,
) -> RenderObjectBundle {
    let mesh = meshes.add(Diamond3d);
    let distance = start.distance(end);
    let transform = Transform::from_translation(start)
        .looking_at(end, Vec3::Z)
        .with_scale(Vec3::new(distance, thickness, thickness));
    RenderObjectBundle {
        mesh,
        material: materials.add(color),
        transform,
        ..Default::default()
    }
}

#[derive(Hash)]
struct Diamond3d;

impl AssetHash<CpuMesh> for Diamond3d {}

impl From<Diamond3d> for CpuMesh {
    fn from(_: Diamond3d) -> Self {
        let girth = 4.0;
        let waist_height = 0.2;

        let ay = f32::to_radians(0.0).sin() * girth;
        let az = f32::to_radians(0.0).cos() * girth;

        let by = f32::to_radians(120.0).sin() * girth;
        let bz = f32::to_radians(120.0).cos() * girth;

        let cy = f32::to_radians(240.0).sin() * girth;
        let cz = f32::to_radians(240.0).cos() * girth;

        let positions = vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(waist_height, ay, az),
            Vec3::new(waist_height, by, bz),
            Vec3::new(waist_height, cy, cz),
            Vec3::new(1.0, 0.0, 0.0),
        ];

        let indices: Indices = Indices(Some(vec![
            0u16, 2, 1, 0, 1, 3, 0, 3, 2, 4, 1, 2, 4, 3, 1, 4, 2, 3,
        ]));

        let mut mesh = CpuMesh {
            indices,
            positions: Positions(positions),
            ..Default::default()
        };

        mesh.compute_normals();

        mesh
    }
}
