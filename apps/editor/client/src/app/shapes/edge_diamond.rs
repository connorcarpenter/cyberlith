use math::Vec3;
use render_api::{
    base::{CpuMaterial, CpuMesh},
    components::{RenderObjectBundle, Transform},
};
use storage::{AssetHash, Handle, Storage};

pub fn create_3d_edge_diamond(
    meshes: &mut Storage<CpuMesh>,
    material: &Handle<CpuMaterial>,
    start: Vec3,
    end: Vec3,
    thickness: f32,
) -> RenderObjectBundle {
    let mesh = meshes.add(Diamond3d);
    let distance = start.distance(end);
    let transform = Transform::from_translation(start)
        .looking_at(end, Vec3::Z)
        .with_scale(Vec3::new(distance, thickness, thickness));
    RenderObjectBundle {
        mesh,
        material: *material,
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

        let indices = vec![0, 2, 1, 0, 1, 3, 0, 3, 2, 4, 1, 2, 4, 3, 1, 4, 2, 3];

        CpuMesh::from_indices(&positions, &indices)
    }
}
