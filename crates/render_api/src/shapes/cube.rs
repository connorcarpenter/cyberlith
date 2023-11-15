use math::{triangle_is_clockwise_toward_camera, Vec3};

use crate::{
    assets::AssetHash,
    base::{CpuMesh, Positions},
};

#[derive(Hash)]
pub struct Cube;

impl AssetHash<CpuMesh> for Cube {}

impl From<Cube> for CpuMesh {
    fn from(_cube: Cube) -> Self {
        let half_size = 1.0;
        let neg_half_size = -1.0;

        let right_top_front = Vec3::new(half_size, half_size, half_size);
        let right_top_back = Vec3::new(half_size, half_size, neg_half_size);
        let right_bottom_front = Vec3::new(half_size, neg_half_size, half_size);
        let right_bottom_back = Vec3::new(half_size, neg_half_size, neg_half_size);

        let left_top_front = Vec3::new(neg_half_size, half_size, half_size);
        let left_top_back = Vec3::new(neg_half_size, half_size, neg_half_size);
        let left_bottom_front = Vec3::new(neg_half_size, neg_half_size, half_size);
        let left_bottom_back = Vec3::new(neg_half_size, neg_half_size, neg_half_size);

        let mut positions = Vec::new();

        // Right Face
        push_quad("right", false, &mut positions, right_top_front, right_bottom_front, right_top_back, right_bottom_back);

        // Left Face
        push_quad("left", true, &mut positions, left_top_front, left_top_back, left_bottom_front, left_bottom_back);

        // Top Face
        push_quad("top", false, &mut positions, right_top_front, right_top_back, left_top_front, left_top_back);

        // Bottom Face
        push_quad("bottom", true, &mut positions, right_bottom_front, left_bottom_front, right_bottom_back, left_bottom_back);

        // Back Face
        push_quad("back", true, &mut positions, right_top_back, right_bottom_back, left_top_back, left_bottom_back);

        // Front Face
        push_quad("front", false, &mut positions, right_top_front, left_top_front, right_bottom_front, left_bottom_front);

        let mut mesh = CpuMesh {
            positions: Positions(positions),
            ..Default::default()
        };
        mesh.compute_normals();
        mesh
    }
}

fn push_quad(name: &str, should_be_cw: bool, positions: &mut Vec<Vec3>, vertex_a: Vec3, vertex_b: Vec3, vertex_c: Vec3, vertex_d: Vec3) {
    push_triangle(name, "1", should_be_cw, positions, vertex_a, vertex_b, vertex_c);
    push_triangle(name, "2", should_be_cw, positions, vertex_c, vertex_b, vertex_d);
}

fn push_triangle(face_name: &str, tri_name: &str, should_be_cw: bool, positions: &mut Vec<Vec3>, vertex_a: Vec3, vertex_b: Vec3, vertex_c: Vec3) {
    let cw = triangle_is_clockwise_toward_camera([vertex_a, vertex_b, vertex_c], Vec3::splat(100.0));
    if should_be_cw != cw {
        panic!("Triangle is not clockwise toward camera: {} {}", face_name, tri_name);
    }
    positions.push(vertex_a);
    positions.push(vertex_b);
    positions.push(vertex_c);
}