use math::{Vec2, Vec3};
use render_api::{
    base::{Color, CpuMaterial, CpuMesh, Indices, Positions},
    components::{RenderObjectBundle, Transform},
    shapes::set_2d_line_transform,
    AssetHash, Assets,
};

pub fn create_2d_edge_line(
    meshes: &mut Assets<CpuMesh>,
    materials: &mut Assets<CpuMaterial>,
    start: Vec2,
    end: Vec2,
    color: Color,
    thickness: f32,
) -> RenderObjectBundle {
    let mesh = meshes.add(Line2d);
    let mut transform = Transform::default();
    transform.scale.y = thickness;
    set_2d_line_transform(&mut transform, start, end);
    RenderObjectBundle {
        mesh,
        material: materials.add(color),
        transform,
        ..Default::default()
    }
}

#[derive(Hash)]
pub struct Line2d;

impl AssetHash<CpuMesh> for Line2d {}

impl From<Line2d> for CpuMesh {
    fn from(_: Line2d) -> Self {
        let positions = vec![
            Vec3::new(0.0, -0.5, 0.0),
            Vec3::new(1.0, -0.5, 0.0),
            Vec3::new(1.0, 0.5, 0.0),
            Vec3::new(0.0, 0.5, 0.0),
        ];

        let indices: Indices = Indices(Some(vec![0u16, 2, 1, 2, 0, 3]));

        let normals = vec![Vec3::Z; 4];

        Self {
            indices,
            positions: Positions(positions),
            normals: Some(normals),
            ..Default::default()
        }
    }
}
