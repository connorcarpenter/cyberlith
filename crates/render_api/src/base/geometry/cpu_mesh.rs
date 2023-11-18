use math::Vec3;

use crate::base::{AxisAlignedBoundingBox, Error, Indices, Positions, Result};

///
/// A CPU-side version of a triangle mesh.
///
#[derive(Clone)]
pub struct CpuMesh {
    /// The positions of the vertices.
    /// If there is no indices associated with this mesh, three contiguous positions defines a triangle, in that case, the length must be divisable by 3.
    pub positions: Positions,
    /// The indices into the positions, normals, uvs and colors arrays which defines the three vertices of a triangle. Three contiguous indices defines a triangle, therefore the length must be divisable by 3.
    pub indices: Indices,
    /// The normals of the vertices.
    pub normals: Option<Vec<Vec3>>,
}

impl Default for CpuMesh {
    fn default() -> Self {
        Self {
            positions: Positions::default(),
            indices: Indices::default(),
            normals: None,
        }
    }
}

impl std::fmt::Debug for CpuMesh {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut d = f.debug_struct("Mesh");
        d.field("positions", &self.positions.len());
        d.field("indices", &self.indices);
        d.field("normals", &self.normals.as_ref().map(|v| v.len()));
        d.finish()
    }
}

impl CpuMesh {
    /// Returns the number of vertices in this mesh.
    pub fn vertex_count(&self) -> usize {
        self.positions.len()
    }

    /// Returns the number of triangles in this mesh.
    pub fn triangle_count(&self) -> usize {
        self.indices
            .len()
            .map(|i| i / 3)
            .unwrap_or(self.positions.len() / 3)
    }

    ///
    /// Computes the per vertex normals and updates the normals of the mesh.
    /// It will override the current normals if they already exist.
    ///
    pub fn compute_normals(&mut self) {
        let mut normals = vec![Vec3::ZERO; self.positions.len()];
        self.for_each_triangle(|i0, i1, i2| {
            let Positions(ref positions) = self.positions;
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
        self.normals = Some(normals);
    }

    ///
    /// Iterates over all triangles in this mesh and calls the callback function with the three indices, one for each vertex in the triangle.
    ///
    pub fn for_each_triangle(&self, mut callback: impl FnMut(usize, usize, usize)) {
        match self.indices {
            Indices(Some(ref indices)) => {
                for face in 0..indices.len() / 3 {
                    let index0 = indices[face * 3] as usize;
                    let index1 = indices[face * 3 + 1] as usize;
                    let index2 = indices[face * 3 + 2] as usize;
                    callback(index0, index1, index2);
                }
            }
            Indices(None) => {
                for face in 0..self.triangle_count() {
                    callback(face * 3, face * 3 + 1, face * 3 + 2);
                }
            }
        }
    }

    ///
    /// Computes the [AxisAlignedBoundingBox] for this triangle mesh.
    ///
    pub fn compute_aabb(&self) -> AxisAlignedBoundingBox {
        self.positions.compute_aabb()
    }

    ///
    /// Returns an error if the mesh is not valid.
    ///
    pub fn validate(&self) -> Result<()> {
        if self.indices.len().map(|i| i % 3 != 0).unwrap_or(false) {
            Err(Error::InvalidNumberOfIndices(self.indices.len().unwrap()))?;
        }
        let vertex_count = self.vertex_count();
        let max_index = match &self.indices {
            Indices(Some(ind)) => ind.iter().max().map(|m| *m as usize),
            Indices(None) => None,
        };
        if max_index.map(|i| i >= vertex_count).unwrap_or(false) {
            Err(Error::InvalidIndices(max_index.unwrap(), vertex_count))?;
        }
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

        buffer_check(Some(self.positions.len()), "position")?;
        buffer_check(self.normals.as_ref().map(|b| b.len()), "normal")?;

        Ok(())
    }
}
