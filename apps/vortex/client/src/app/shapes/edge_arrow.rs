use math::{Vec2, Vec3};
use render_api::{
    base::{Color, CpuMaterial, CpuMesh, Indices, Positions},
    components::{RenderObjectBundle, Transform},
    shapes::set_2d_line_transform,
    AssetHash, Assets,
};

pub fn create_2d_edge_arrow(
    meshes: &mut Assets<CpuMesh>,
    materials: &mut Assets<CpuMaterial>,
    start: Vec2,
    end: Vec2,
    color: Color,
) -> RenderObjectBundle {
    let mesh = meshes.add(Arrow);
    let mut transform = Transform::default();
    set_2d_line_transform(&mut transform, start, end);
    RenderObjectBundle {
        mesh,
        material: materials.add(color),
        transform,
        ..Default::default()
    }
}

#[derive(Hash)]
pub struct Arrow;

impl AssetHash<CpuMesh> for Arrow {}

impl From<Arrow> for CpuMesh {
    fn from(_: Arrow) -> Self {
        let head_base_x = 0.85;
        let head_point_x = 0.84;
        let head_width = 2.5;
        let neg_head_width = head_width * -1.0;

        let positions = vec![
            Vec3::new(0.0, -0.5, 0.0),
            Vec3::new(head_base_x, -0.5, 0.0),
            Vec3::new(head_base_x, 0.5, 0.0),
            Vec3::new(0.0, 0.5, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(head_point_x, neg_head_width, 0.0),
            Vec3::new(head_point_x, head_width, 0.0),
        ];

        let indices: Indices = Indices(Some(vec![0u16, 1, 2, 2, 3, 0, 4, 1, 2, 4, 1, 5, 4, 2, 6]));

        let normals = vec![Vec3::Z; 7];

        Self {
            indices,
            positions: Positions(positions),
            normals: Some(normals),
            ..Default::default()
        }
    }
}
