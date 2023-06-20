use math::*;

use crate::base::CpuTexture2D;

#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
///
/// The 6 sides of a cube map
///
pub enum CubeSide {
    /// Positive y
    Top,
    /// Negative y
    Bottom,
    /// Positive x
    Right,
    /// Negative x
    Left,
    /// Negative z
    Front,
    /// Positive z
    Back,
}

///
/// Iterator over the 6 side of a cube map.
///
pub struct CubeSideIterator {
    index: usize,
}

impl CubeSideIterator {
    ///
    /// Creates a new iterator over the 6 side of a cube map.
    ///
    pub fn new() -> Self {
        Self { index: 0 }
    }
}

impl Default for CubeSideIterator {
    fn default() -> Self {
        Self::new()
    }
}

impl Iterator for CubeSideIterator {
    type Item = CubeSide;
    fn next(&mut self) -> Option<Self::Item> {
        self.index += 1;
        match self.index {
            1 => Some(CubeSide::Right),
            2 => Some(CubeSide::Left),
            3 => Some(CubeSide::Top),
            4 => Some(CubeSide::Bottom),
            5 => Some(CubeSide::Front),
            6 => Some(CubeSide::Back),
            _ => None,
        }
    }
}

impl CubeSide {
    ///
    /// Iterator over the 6 side of a cube map.
    ///
    pub fn iter() -> CubeSideIterator {
        CubeSideIterator::new()
    }

    /// The up direction that should be used when rendering into this cube map side.
    pub fn up(&self) -> Vec3 {
        match self {
            CubeSide::Right => Vec3::new(0.0, -1.0, 0.0),
            CubeSide::Left => Vec3::new(0.0, -1.0, 0.0),
            CubeSide::Top => Vec3::Z,
            CubeSide::Bottom => Vec3::new(0.0, 0.0, -1.0),
            CubeSide::Front => Vec3::new(0.0, -1.0, 0.0),
            CubeSide::Back => Vec3::new(0.0, -1.0, 0.0),
        }
    }

    /// The direction from origo towards the center of this cube map side.
    pub fn direction(&self) -> Vec3 {
        match self {
            CubeSide::Right => Vec3::X,
            CubeSide::Left => Vec3::new(-1.0, 0.0, 0.0),
            CubeSide::Top => Vec3::Y,
            CubeSide::Bottom => Vec3::new(0.0, -1.0, 0.0),
            CubeSide::Front => Vec3::Z,
            CubeSide::Back => Vec3::new(0.0, 0.0, -1.0),
        }
    }
}

pub struct CpuTextureCube {
    pub right: CpuTexture2D,
    pub left: CpuTexture2D,
    pub top: CpuTexture2D,
    pub bottom: CpuTexture2D,
    pub front: CpuTexture2D,
    pub back: CpuTexture2D,
}

impl CpuTextureCube {
    pub fn new(
        right: CpuTexture2D,
        left: CpuTexture2D,
        top: CpuTexture2D,
        bottom: CpuTexture2D,
        front: CpuTexture2D,
        back: CpuTexture2D,
    ) -> Self {
        Self {
            right,
            left,
            top,
            bottom,
            front,
            back,
        }
    }
}
