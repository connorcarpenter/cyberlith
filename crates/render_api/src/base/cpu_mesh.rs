
use math::Vec3;

use crate::base::AxisAlignedBoundingBox;

///
/// A CPU-side version of a triangle mesh.
///
#[derive(Clone)]
pub struct CpuMesh {
    /// The positions of the vertices.
    /// If there is no indices associated with this mesh, three contiguous positions defines a triangle, in that case, the length must be divisable by 3.
    vertices: Vec<Vec3>,
}

impl Default for CpuMesh {
    fn default() -> Self {
        Self {
            vertices: Vec::new(),
        }
    }
}

impl std::fmt::Debug for CpuMesh {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut d = f.debug_struct("CpuMesh");
        d.field("vertices", &self.vertices.len());
        d.finish()
    }
}

impl CpuMesh {
    pub fn from_vertices(vertices: Vec<Vec3>) -> Self {
        Self { vertices }
    }

    pub fn from_indices(vertices: &[Vec3], indices: &[usize]) -> Self {
        let mut new_vertices = Vec::new();
        for index in indices {
            new_vertices.push(vertices[*index]);
        }
        Self::from_vertices(new_vertices)
    }

    pub fn to_vertices(&self) -> Vec<Vec3> {
        self.vertices.clone()
    }

    /// Returns the number of vertices in this mesh.
    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    /// Returns the number of triangles in this mesh.
    pub fn triangle_count(&self) -> usize {
        self.vertices.len() / 3
    }

    ///
    /// Iterates over all triangles in this mesh and calls the callback function with the three indices, one for each vertex in the triangle.
    ///
    pub fn for_each_triangle(&self, mut callback: impl FnMut(usize, usize, usize)) {
        for face in 0..self.triangle_count() {
            callback(face * 3, face * 3 + 1, face * 3 + 2);
        }
    }

    ///
    /// Computes the [AxisAlignedBoundingBox] for this triangle mesh.
    ///
    pub fn compute_aabb(&self) -> AxisAlignedBoundingBox {
        AxisAlignedBoundingBox::new_with_positions(&self.vertices)
    }

    ///
    /// Computes the per vertex normals and updates the normals of the mesh.
    /// It will override the current normals if they already exist.
    ///
    pub fn compute_normals(&self) -> Vec<Vec3> {
        let mut normals = Vec::new();
        self.for_each_triangle(|i0, i1, i2| {
            let normal = {
                let p0 = self.vertices[i0];
                let p1 = self.vertices[i1];
                let p2 = self.vertices[i2];
                (p1 - p0).cross(p2 - p0)
            };
            normals.push(normal);
            normals.push(normal);
            normals.push(normal);
        });

        normals
    }
}