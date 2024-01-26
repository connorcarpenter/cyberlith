use render_api::components::{Camera, Projection, Transform};

// Render Camera
#[derive(Clone, Copy)]
pub struct RenderCamera<'a> {
    pub camera: &'a Camera,
    pub transform: &'a Transform,
    pub projection: &'a Projection,
}

impl<'a> RenderCamera<'a> {
    pub fn new(camera: &'a Camera, transform: &'a Transform, projection: &'a Projection) -> Self {
        Self {
            camera,
            transform,
            projection,
        }
    }
}
