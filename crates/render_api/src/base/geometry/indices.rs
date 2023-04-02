///
/// An array of indices. Supports different data types.
///
#[derive(Clone, Debug)]
pub struct Indices(pub Option<Vec<u16>>);

impl Indices {
    ///
    /// Converts all the indices as `u32` data type.
    ///
    pub fn into_u32(self) -> Option<Vec<u32>> {
        match self.0 {
            None => None,
            Some(mut values) => Some(values.drain(..).map(|i| i as u32).collect::<Vec<_>>()),
        }
    }

    ///
    /// Clones and converts all the indices as `u32` data type.
    ///
    pub fn to_u32(&self) -> Option<Vec<u32>> {
        match &self.0 {
            None => None,
            Some(values) => Some(values.iter().map(|i| *i as u32).collect::<Vec<_>>()),
        }
    }

    ///
    /// Returns the number of indices.
    ///
    pub fn len(&self) -> Option<usize> {
        match self {
            Self(None) => None,
            Self(Some(values)) => Some(values.len()),
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
        Self(None)
    }
}
