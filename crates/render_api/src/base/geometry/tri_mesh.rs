use math::*;

use crate::{
    base::{AxisAlignedBoundingBox, Color, Error, Indices, Positions, Result},
    components::Transform,
};

///
/// A CPU-side version of a triangle mesh.
///
#[derive(Clone)]
pub struct TriMesh {
    /// The positions of the vertices.
    /// If there is no indices associated with this mesh, three contiguous positions defines a triangle, in that case, the length must be divisable by 3.
    pub positions: Positions,
    /// The indices into the positions, normals, uvs and colors arrays which defines the three vertices of a triangle. Three contiguous indices defines a triangle, therefore the length must be divisable by 3.
    pub indices: Indices,
    /// The normals of the vertices.
    pub normals: Option<Vec<Vec3>>,
    /// The tangents of the vertices, orthogonal direction to the normal.
    /// The fourth value specifies the handedness (either -1.0 or 1.0).
    pub tangents: Option<Vec<Vec4>>,
    /// The uv coordinates of the vertices.
    pub uvs: Option<Vec<Vec2>>,
    /// The colors of the vertices.
    /// The colors are assumed to be in linear space.
    pub colors: Option<Vec<Color>>,
}

impl std::default::Default for TriMesh {
    fn default() -> Self {
        Self {
            positions: Positions::default(),
            indices: Indices::default(),
            normals: None,
            tangents: None,
            uvs: None,
            colors: None,
        }
    }
}

impl std::fmt::Debug for TriMesh {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut d = f.debug_struct("Mesh");
        d.field("positions", &self.positions.len());
        d.field("indices", &self.indices);
        d.field("normals", &self.normals.as_ref().map(|v| v.len()));
        d.field("tangents", &self.tangents.as_ref().map(|v| v.len()));
        d.field("uvs", &self.uvs.as_ref().map(|v| v.len()));
        d.field("colors", &self.colors.as_ref().map(|v| v.len()));
        d.finish()
    }
}

impl TriMesh {
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
    /// Transforms the mesh by the given transformation.
    ///
    pub fn transform_mat4(&mut self, mat4: &Mat4) -> Result<()> {
        let Positions(positions) = &mut self.positions;
        for pos in positions.iter_mut() {
            *pos = (*mat4 * pos.extend(1.0)).truncate();
        }

        if self.normals.is_some() || self.tangents.is_some() {
            let normal_transform = mat4.inverse().transpose();

            if let Some(ref mut normals) = self.normals {
                for n in normals.iter_mut() {
                    *n = (normal_transform * n.extend(1.0)).truncate();
                }
            }
            if let Some(ref mut tangents) = self.tangents {
                for t in tangents.iter_mut() {
                    *t = (normal_transform * t.truncate().extend(1.0))
                        .truncate()
                        .extend(t.w);
                }
            }
        }
        Ok(())
    }

    pub fn transform(&mut self, transform: &Transform) -> Result<()> {
        let mat4 = transform.compute_matrix();
        self.transform_mat4(&mat4)
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
    /// Computes the per vertex tangents and updates the tangents of the mesh.
    /// It will override the current tangents if they already exist.
    ///
    pub fn compute_tangents(&mut self) {
        if self.normals.is_none() || self.uvs.is_none() {
            panic!("mesh must have both normals and uv coordinates to be able to compute tangents");
        }
        let mut tan1 = vec![Vec3::ZERO; self.positions.len()];
        let mut tan2 = vec![Vec3::ZERO; self.positions.len()];

        self.for_each_triangle(|i0, i1, i2| {
            let Positions(positions) = &self.positions;
            let (a, b, c) = (positions[i0], positions[i1], positions[i2]);
            let uva = self.uvs.as_ref().unwrap()[i0];
            let uvb = self.uvs.as_ref().unwrap()[i1];
            let uvc = self.uvs.as_ref().unwrap()[i2];

            let ba = b - a;
            let ca = c - a;

            let uvba = uvb - uva;
            let uvca = uvc - uva;

            let d = uvba.x * uvca.y - uvca.x * uvba.y;
            if d.abs() > 0.00001 {
                let r = 1.0 / d;
                let sdir = (ba * uvca.y - ca * uvba.y) * r;
                let tdir = (ca * uvba.x - ba * uvca.x) * r;
                tan1[i0] += sdir;
                tan1[i1] += sdir;
                tan1[i2] += sdir;
                tan2[i0] += tdir;
                tan2[i1] += tdir;
                tan2[i2] += tdir;
            }
        });

        let mut tangents = vec![Vec4::new(0.0, 0.0, 0.0, 0.0); self.positions.len()];
        self.for_each_vertex(|index| {
            let normal = self.normals.as_ref().unwrap()[index];
            let t = tan1[index];
            let tangent = (t - normal * normal.dot(t)).normalize();
            let handedness = if normal.cross(tangent).dot(tan2[index]) < 0.0 {
                1.0
            } else {
                -1.0
            };
            tangents[index] = tangent.extend(handedness);
        });

        self.tangents = Some(tangents);
    }

    ///
    ///  Iterates over all vertices in this mesh and calls the callback function with the index for each vertex.
    ///
    pub fn for_each_vertex(&self, mut callback: impl FnMut(usize)) {
        for i in 0..self.positions.len() {
            callback(i);
        }
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
        buffer_check(self.tangents.as_ref().map(|b| b.len()), "tangent")?;
        buffer_check(self.colors.as_ref().map(|b| b.len()), "color")?;
        buffer_check(self.uvs.as_ref().map(|b| b.len()), "uv coordinate")?;

        Ok(())
    }
}
