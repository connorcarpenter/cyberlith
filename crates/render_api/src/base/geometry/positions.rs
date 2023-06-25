use math::*;

use crate::base::AxisAlignedBoundingBox;

///
/// An array of positions
///
#[derive(Clone)]
pub struct Positions(pub Vec<Vec3>);

impl Positions {
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

impl std::default::Default for Positions {
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl std::fmt::Debug for Positions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut d = f.debug_struct("Positions");
        d.field("f32", &self.0.len());
        d.finish()
    }
}
