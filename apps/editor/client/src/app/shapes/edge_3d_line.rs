use math::Vec3;
use render_api::{
    base::{CpuMaterial, CpuMesh},
    components::{RenderObjectBundle, Transform},
};
use storage::{Handle, Storage, StorageHash};

pub fn create_3d_edge_line(
    meshes: &mut Storage<CpuMesh>,
    material: &Handle<CpuMaterial>,
    start: Vec3,
    end: Vec3,
    thickness: f32,
) -> RenderObjectBundle {
    let mesh = meshes.add(Line3d);
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
struct Line3d;

impl StorageHash<CpuMesh> for Line3d {}

impl From<Line3d> for CpuMesh {
    fn from(_: Line3d) -> Self {
        let girth = 2.0;

        let ay = f32::to_radians(0.0).sin() * girth;
        let az = f32::to_radians(0.0).cos() * girth;

        let by = f32::to_radians(120.0).sin() * girth;
        let bz = f32::to_radians(120.0).cos() * girth;

        let cy = f32::to_radians(240.0).sin() * girth;
        let cz = f32::to_radians(240.0).cos() * girth;

        let positions = vec![
            Vec3::new(0.0, ay, az),
            Vec3::new(0.0, by, bz),
            Vec3::new(0.0, cy, cz),
            Vec3::new(1.0, ay, az),
            Vec3::new(1.0, by, bz),
            Vec3::new(1.0, cy, cz),
        ];

        let indices = vec![
            0, 2, 1, 3, 5, 4, 0, 1, 3, 3, 1, 4, 1, 2, 4, 4, 2, 5, 2, 0, 5, 5, 0, 3,
        ];

        // or
        //         let indices: Indices = Indices(Some(vec![
        //             0u16, 1, 2,
        //             3, 4, 5,
        //             0, 3, 1,
        //             3, 4, 1,
        //             1, 4, 2,
        //             4, 5, 2,
        //             2, 5, 0,
        //             5, 3, 0,
        //         ]));

        CpuMesh::from_indices(&positions, &indices)
    }
}
