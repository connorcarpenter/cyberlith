use math::{Vec2, Vec3};
use render_api::{
    base::{CpuMaterial, CpuMesh},
    components::{RenderObjectBundle, Transform},
    shapes::set_2d_line_transform,
};
use storage::{AssetHash, Handle, Storage};

pub fn create_2d_edge_arrow(
    meshes: &mut Storage<CpuMesh>,
    material: &Handle<CpuMaterial>,
    start: Vec2,
    end: Vec2,
    depth: f32,
    thickness: f32,
    arrow_head_width: f32,
) -> RenderObjectBundle {
    let mesh = meshes.add(Arrow2d::new(arrow_head_width));
    let mut transform = Transform::default();
    transform.scale.y = thickness;
    set_2d_line_transform(&mut transform, start, end, depth);
    RenderObjectBundle {
        mesh,
        material: *material,
        transform,
        ..Default::default()
    }
}

#[derive(Hash)]
pub struct Arrow2d {
    pub head_width_tenths: u8,
}

impl Default for Arrow2d {
    fn default() -> Self {
        Self {
            head_width_tenths: 20,
        }
    }
}

impl Arrow2d {
    pub fn new(head_width: f32) -> Self {
        Self {
            head_width_tenths: ((head_width * 10.0) as u8),
        }
    }
}

impl AssetHash<CpuMesh> for Arrow2d {}

impl From<Arrow2d> for CpuMesh {
    fn from(arrow_2d: Arrow2d) -> Self {
        let head_base_x = 0.8;
        let head_point_x = 0.79;
        let head_width = (arrow_2d.head_width_tenths as f32) / 10.0;
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

        let indices = vec![0, 2, 1, 2, 0, 3, 4, 1, 2, 4, 5, 1, 4, 2, 6];

        Self::from_indices(&positions, &indices)
    }
}
