use math::{triangle_is_ccw_toward_point, Vec3};
use storage::StorageHash;

use crate::base::CpuMesh;

#[derive(Hash)]
pub struct Cube;

impl StorageHash<CpuMesh> for Cube {}

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
        push_quad(
            "right",
            true,
            &mut positions,
            right_top_front,
            right_top_back,
            right_bottom_front,
            right_bottom_back,
        );

        // Left Face
        push_quad(
            "left",
            false,
            &mut positions,
            left_top_front,
            left_bottom_front,
            left_top_back,
            left_bottom_back,
        );

        // Top Face
        push_quad(
            "top",
            true,
            &mut positions,
            right_top_front,
            left_top_front,
            right_top_back,
            left_top_back,
        );

        // Bottom Face
        push_quad(
            "bottom",
            false,
            &mut positions,
            right_bottom_front,
            right_bottom_back,
            left_bottom_front,
            left_bottom_back,
        );

        // Back Face
        push_quad(
            "back",
            false,
            &mut positions,
            right_top_back,
            left_top_back,
            right_bottom_back,
            left_bottom_back,
        );

        // Front Face
        push_quad(
            "front",
            true,
            &mut positions,
            right_top_front,
            right_bottom_front,
            left_top_front,
            left_bottom_front,
        );

        CpuMesh::from_vertices(positions)
    }
}

fn push_quad(
    name: &str,
    should_be_ccw: bool,
    positions: &mut Vec<Vec3>,
    vertex_a: Vec3,
    vertex_b: Vec3,
    vertex_c: Vec3,
    vertex_d: Vec3,
) {
    push_triangle(
        name,
        "1",
        should_be_ccw,
        positions,
        vertex_a,
        vertex_b,
        vertex_c,
    );
    push_triangle(
        name,
        "2",
        should_be_ccw,
        positions,
        vertex_c,
        vertex_b,
        vertex_d,
    );
}

fn push_triangle(
    face_name: &str,
    tri_name: &str,
    should_be_ccw: bool,
    positions: &mut Vec<Vec3>,
    vertex_a: Vec3,
    vertex_b: Vec3,
    vertex_c: Vec3,
) {
    if let Some(ccw) =
        triangle_is_ccw_toward_point([vertex_a, vertex_b, vertex_c], Vec3::splat(100.0))
    {
        if should_be_ccw != ccw {
            panic!(
                "Triangle is not clockwise toward camera: {} {}",
                face_name, tri_name
            );
        }
    }
    positions.push(vertex_a);
    positions.push(vertex_b);
    positions.push(vertex_c);
}
