use math::Vec2;
use render_api::{
    base::{CpuMaterial, CpuMesh},
    components::{RenderObjectBundle, Transform},
    shapes::{set_2d_line_transform, Line},
};
use storage::{Assets, Handle};

pub fn create_2d_edge_line(
    meshes: &mut Assets<CpuMesh>,
    material: &Handle<CpuMaterial>,
    start: Vec2,
    end: Vec2,
    depth: f32,
    thickness: f32,
) -> RenderObjectBundle {
    let mesh = meshes.add(Line);
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
