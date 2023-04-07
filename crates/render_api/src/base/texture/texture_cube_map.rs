use math::*;

use crate::base::Texture2D;

#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
///
/// The 6 sides of a cube map
///
pub enum CubeMapSide {
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
pub struct CubeMapSideIterator {
    index: usize,
}

impl CubeMapSideIterator {
    ///
    /// Creates a new iterator over the 6 side of a cube map.
    ///
    pub fn new() -> Self {
        Self { index: 0 }
    }
}

impl Default for CubeMapSideIterator {
    fn default() -> Self {
        Self::new()
    }
}

impl Iterator for CubeMapSideIterator {
    type Item = CubeMapSide;
    fn next(&mut self) -> Option<Self::Item> {
        self.index += 1;
        match self.index {
            1 => Some(CubeMapSide::Right),
            2 => Some(CubeMapSide::Left),
            3 => Some(CubeMapSide::Top),
            4 => Some(CubeMapSide::Bottom),
            5 => Some(CubeMapSide::Front),
            6 => Some(CubeMapSide::Back),
            _ => None,
        }
    }
}

impl CubeMapSide {
    ///
    /// Iterator over the 6 side of a cube map.
    ///
    pub fn iter() -> CubeMapSideIterator {
        CubeMapSideIterator::new()
    }

    /// The up direction that should be used when rendering into this cube map side.
    pub fn up(&self) -> Vec3 {
        match self {
            CubeMapSide::Right =>  Vec3::new(0.0, -1.0, 0.0),
            CubeMapSide::Left =>   Vec3::new(0.0, -1.0, 0.0),
            CubeMapSide::Top =>    Vec3::new(0.0, 0.0, 1.0),
            CubeMapSide::Bottom => Vec3::new(0.0, 0.0, -1.0),
            CubeMapSide::Front =>  Vec3::new(0.0, -1.0, 0.0),
            CubeMapSide::Back =>   Vec3::new(0.0, -1.0, 0.0),
        }
    }

    /// The direction from origo towards the center of this cube map side.
    pub fn direction(&self) -> Vec3 {
        match self {
            CubeMapSide::Right =>  Vec3::new(1.0, 0.0, 0.0),
            CubeMapSide::Left =>   Vec3::new(-1.0, 0.0, 0.0),
            CubeMapSide::Top =>    Vec3::new(0.0, 1.0, 0.0),
            CubeMapSide::Bottom => Vec3::new(0.0, -1.0, 0.0),
            CubeMapSide::Front =>  Vec3::new(0.0, 0.0, 1.0),
            CubeMapSide::Back =>   Vec3::new(0.0, 0.0, -1.0),
        }
    }
}

pub struct TextureCubeMap {
    pub right: Texture2D,
    pub left: Texture2D,
    pub top: Texture2D,
    pub bottom: Texture2D,
    pub front: Texture2D,
    pub back: Texture2D,
}

impl TextureCubeMap {
    pub fn new(
        right: Texture2D,
        left: Texture2D,
        top: Texture2D,
        bottom: Texture2D,
        front: Texture2D,
        back: Texture2D,
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
