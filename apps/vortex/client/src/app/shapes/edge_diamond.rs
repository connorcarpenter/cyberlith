
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
) -> RenderObjectBundle {
    let mesh = meshes.add(Edge3dMesh);
    let distance = start.distance(end);
    let transform = Transform::from_translation(start)
        .looking_at(end, Vec3::Y)
        .with_scale(Vec3::new(distance, 1.0, 1.0));
    RenderObjectBundle {
        mesh,
        material: materials.add(color),
        transform,
        ..Default::default()
    }
}

#[derive(Hash)]
struct Edge3dMesh;

impl AssetHash<CpuMesh> for Edge3dMesh {}

impl From<Edge3dMesh> for CpuMesh {
    fn from(_: Edge3dMesh) -> Self {
        create_3d_edge_diamond_mesh()
    }
}

fn create_3d_edge_diamond_mesh() -> CpuMesh {
    let girth = 6.0;
    let waist_height = 0.9;

    let ax = 1.0 * girth;
    let ay = 0.0 * girth;

    let bx = -1.0 * girth;
    let by = 1.0 * girth;

    let cx = -1.0 * girth;
    let cy = -1.0 * girth;

    let positions = vec![
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(ax, ay, waist_height),
        Vec3::new(bx, by, waist_height),
        Vec3::new(cx, cy, waist_height),
        Vec3::new(0.0, 0.0, 1.0),
    ];

    let indices: Indices = Indices(Some(vec![
        0u16, 2, 1, 0, 3, 1, 0, 2, 3, 4, 1, 2, 4, 1, 3, 4, 3, 2,
    ]));

    let mut mesh = CpuMesh {
        indices,
        positions: Positions(positions),
        ..Default::default()
    };

    mesh.compute_normals();

    mesh
}
