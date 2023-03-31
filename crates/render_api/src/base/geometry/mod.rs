//!
//! Contain geometry asset definitions.
//!

mod tri_mesh;
pub use tri_mesh::*;

use cgmath::*;

use crate::base::{AxisAlignedBoundingBox, Vec3};

///
/// An array of indices. Supports different data types.
///
#[derive(Clone, Debug)]
pub enum Indices {
    /// Uses unsigned 16 bit integer for each index.
    U16(Vec<u16>),
}

impl Indices {
    ///
    /// Converts all the indices as `u32` data type.
    ///
    pub fn into_u32(self) -> Option<Vec<u32>> {
        match self {
            Self::U16(mut values) => Some(values.drain(..).map(|i| i as u32).collect::<Vec<_>>()),
        }
    }

    ///
    /// Clones and converts all the indices as `u32` data type.
    ///
    pub fn to_u32(&self) -> Option<Vec<u32>> {
        match self {
            Self::U16(values) => Some(values.iter().map(|i| *i as u32).collect::<Vec<_>>()),
        }
    }

    ///
    /// Returns the number of indices.
    ///
    pub fn len(&self) -> Option<usize> {
        match self {
            Self::U16(values) => Some(values.len()),
        }
    }

    ///
    /// Returns whether the set of indices is empty.
    ///
    pub fn is_empty(&self) -> bool {
        self.len().map(|i| i == 0).unwrap_or(true)
    }
}

impl std::default::Default for Indices {
    fn default() -> Self {
        Self::U16(Vec::new())
    }
}

///
/// An array of positions. Supports f32 and f64 data types.
///
#[derive(Clone)]
pub enum Positions {
    F32(Vec<Vec3>),
}

impl Positions {
    ///
    /// Converts and returns all the positions as `f32` data type.
    ///
    pub fn into_f32(self) -> Vec<Vec3> {
        match self {
            Self::F32(values) => values,
        }
    }

    ///
    /// Clones and converts all the positions as `f32` data type.
    ///
    pub fn to_f32(&self) -> Vec<Vec3> {
        match self {
            Self::F32(values) => values.clone(),
        }
    }
    ///
    /// Converts and returns all the positions as `f64` data type.
    ///
    pub fn into_f64(self) -> Vec<Vector3<f64>> {
        match self {
            Self::F32(mut values) => values
                .drain(..)
                .map(|v| Vector3::new(v.x as f64, v.y as f64, v.z as f64))
                .collect::<Vec<_>>(),
        }
    }

    ///
    /// Clones and converts all the positions as `f64` data type.
    ///
    pub fn to_f64(&self) -> Vec<Vector3<f64>> {
        match self {
            Self::F32(values) => values
                .iter()
                .map(|v| Vector3::new(v.x as f64, v.y as f64, v.z as f64))
                .collect::<Vec<_>>(),
        }
    }

    ///
    /// Returns the number of positions.
    ///
    pub fn len(&self) -> usize {
        match self {
            Self::F32(values) => values.len(),
        }
    }

    ///
    /// Returns whether the set of positions is empty.
    ///
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    ///
    /// Computes the [AxisAlignedBoundingBox] for these positions.
    ///
    pub fn compute_aabb(&self) -> AxisAlignedBoundingBox {
        match self {
            Positions::F32(ref positions) => AxisAlignedBoundingBox::new_with_positions(positions),
        }
    }
}

impl std::default::Default for Positions {
    fn default() -> Self {
        Self::F32(Vec::new())
    }
}

impl std::fmt::Debug for Positions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut d = f.debug_struct("Positions");
        match self {
            Self::F32(ind) => d.field("f32", &ind.len()),
        };
        d.finish()
    }
}
