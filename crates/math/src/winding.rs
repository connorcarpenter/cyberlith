use glam::Vec3;
use logging::warn;

// returns whether last two vertices were swapped
pub fn reorder_triangle_winding(
    vertices: &mut [Vec3; 3],
    external_point: Vec3,
    set_to_clockwise: bool,
) -> bool {
    let Some(currently_ccw) = triangle_is_ccw_toward_point(*vertices, external_point) else {
        return false;
    };
    let currently_clockwise = !currently_ccw;

    if currently_clockwise == set_to_clockwise {
        return false;
    }

    // swap vertices
    vertices.swap(1, 2);
    return true;
}

// returns whether last two vertices were swapped
pub fn triangle_is_ccw_toward_point(vertices: [Vec3; 3], external_point: Vec3) -> Option<bool> {
    let a = vertices[0];
    let b = vertices[1];
    let c = vertices[2];

    let ab = b - a;
    let ac = c - a;
    let normal = ab.cross(ac);

    let v = external_point - a;

    let dot_product = normal.dot(v);

    return if dot_product > 0.0 {
        // is counter-clockwise order
        Some(false)
    } else if dot_product < 0.0 {
        // is clockwise order
        Some(true)
    } else {
        warn!("coplanar!");
        None
    };
}
