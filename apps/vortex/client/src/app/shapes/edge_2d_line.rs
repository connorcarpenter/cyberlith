use math::Vec2;
use render_api::{
    base::{Color, CpuMaterial, CpuMesh},
    components::{RenderObjectBundle, Transform},
    shapes::{set_2d_line_transform, Line},
    Assets,
};

pub fn create_2d_edge_line(
    meshes: &mut Assets<CpuMesh>,
    materials: &mut Assets<CpuMaterial>,
    start: Vec2,
    end: Vec2,
    depth: f32,
    color: Color,
    thickness: f32,
) -> RenderObjectBundle {
    let mesh = meshes.add(Line);
    let mut transform = Transform::default();
    transform.scale.y = thickness;
    set_2d_line_transform(&mut transform, start, end, depth);
    RenderObjectBundle {
        mesh,
        material: materials.add(color),
        transform,
        ..Default::default()
    }
}
