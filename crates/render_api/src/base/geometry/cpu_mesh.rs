use bevy_log::info;

use math::Vec3;

use crate::base::{AxisAlignedBoundingBox, Error, Vertices, Result};

///
/// A CPU-side version of a triangle mesh.
///
#[derive(Clone)]
pub struct CpuMesh {
    /// The positions of the vertices.
    /// If there is no indices associated with this mesh, three contiguous positions defines a triangle, in that case, the length must be divisable by 3.
    pub vertices: Vertices,
}

impl Default for CpuMesh {
    fn default() -> Self {
        Self {
            vertices: Vertices::default(),
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
    /// Returns the number of vertices in this mesh.
    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    /// Returns the number of triangles in this mesh.
    pub fn triangle_count(&self) -> usize {
        self.vertices.len() / 3
    }

    ///
    /// Computes the per vertex normals and updates the normals of the mesh.
    /// It will override the current normals if they already exist.
    ///
    pub fn compute_normals(&self) -> Vec<Vec3> {
        let mut normals = vec![Vec3::ZERO; self.vertices.len()];
        self.for_each_triangle(|i0, i1, i2| {
            let Vertices(ref positions) = self.vertices;
            let normal = {
                let p0 = positions[i0];
                let p1 = positions[i1];
                let p2 = positions[i2];
                (p1 - p0).cross(p2 - p0)
            };
            normals[i0] += normal;
            normals[i1] += normal;
            normals[i2] += normal;
        });

        for n in normals.iter_mut() {
            *n = n.normalize();
        }

        normals
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
        self.vertices.compute_aabb()
    }

    ///
    /// Returns an error if the mesh is not valid.
    ///
    pub fn validate(&self) -> Result<()> {
        let vertex_count = self.vertex_count();
        let positions = self.vertices.len();
        info!("validating mesh with {} vertices and {} positions", vertex_count, positions);
        let buffer_check = |length: Option<usize>, name: &str| -> Result<()> {
            if let Some(length) = length {
                if length < vertex_count {
                    Err(Error::InvalidBufferLength(
                        name.to_string(),
                        vertex_count,
                        length,
                    ))?;
                }
            }
            Ok(())
        };

        buffer_check(Some(self.vertices.len()), "position")?;

        Ok(())
    }
}
