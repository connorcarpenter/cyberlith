use crate::base::{Positions, TriMesh};

pub struct Cube {
    pub size: f32,
}

impl From<Cube> for TriMesh {
    fn from(cube: Cube) -> Self {
        let size = cube.size;
        let half_size = size / 2.0;

        let mut tri_mesh = Self::cube();

        let Positions::F32(positions) = &mut tri_mesh.positions else {
            panic!("Should not happen");
        };
        for vertex in positions {
            vertex.x *= half_size;
            vertex.y *= half_size;
            vertex.z *= half_size;
        }

        tri_mesh
    }
}
