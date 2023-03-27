use std::default::Default;

use crate::shape::{Cube, Plane};

#[derive(Default)]
pub struct Mesh {

}

impl From<Plane> for Mesh {
    fn from(plane: Plane) -> Self {
        Self {

        }
    }
}

impl From<Cube> for Mesh {
    fn from(cube: Cube) -> Self {
        Self {

        }
    }
}