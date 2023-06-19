use crate::{
    base::TriMesh,
    shapes::Rectangle,
};

pub struct Square {
    pub size: f32,
}

impl Square {
    pub fn new(size: f32) -> Self {
        Self { size }
    }
}

impl From<Square> for TriMesh {
    fn from(square: Square) -> Self {
        let rect = Rectangle::new(square.size, square.size);
        TriMesh::from(rect)
    }
}
