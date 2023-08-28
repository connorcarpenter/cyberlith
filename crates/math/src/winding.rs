use glam::Vec3;

// returns whether last two vertices were swapped
pub fn reorder_triangle_winding(vertices: &mut [Vec3; 3], external_point: Vec3, set_to_clockwise: bool) -> bool {
    let a= vertices[0];
    let b = vertices[1];
    let c = vertices[2];

    let ab = b - a;
    let ac = c - a;
    let normal = ab.cross(ac);

    let v = external_point - a;

    let dot_product = normal.dot(v);

    let currently_clockwise = if dot_product > 0.0 {
        // is counter-clockwise order
        false
    } else if dot_product < 0.0 {
        // is clockwise order
        true
    } else {
        // is coplanar
        return false;
    };

    if currently_clockwise == set_to_clockwise {
        return false;
    }

    // swap vertices
    vertices.swap(1, 2);
    return true;
}