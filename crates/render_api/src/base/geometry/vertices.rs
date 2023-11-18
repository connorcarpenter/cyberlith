use math::Vec3;

use crate::base::AxisAlignedBoundingBox;

///
/// An array of positions
///
#[derive(Clone)]
pub struct Vertices(pub Vec<Vec3>);

impl Vertices {

    pub fn from_indices(vertices: &[Vec3], indices: &[usize]) -> Self {
        let mut new_positions = Vec::new();
        for index in indices {
            new_positions.push(vertices[*index]);
        }
        Self(new_positions)
    }

    ///
    /// Converts and returns all the positions as `f32` data type.
    ///
    pub fn into_f32(self) -> Vec<Vec3> {
        self.0
    }

    ///
    /// Clones and converts all the positions as `f32` data type.
    ///
    pub fn to_f32(&self) -> Vec<Vec3> {
        self.0.clone()
    }

    ///
    /// Returns the number of positions.
    ///
    pub fn len(&self) -> usize {
        self.0.len()
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
        AxisAlignedBoundingBox::new_with_positions(&self.0)
    }
}

impl Default for Vertices {
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl std::fmt::Debug for Vertices {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut d = f.debug_struct("Vertices");
        d.field("f32", &self.0.len());
        d.finish()
    }
}
