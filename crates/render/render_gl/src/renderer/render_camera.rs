use render_api::components::{Camera, Projection, Transform};

// Render Camera
#[derive(Clone, Copy)]
pub struct RenderCamera {
    pub camera: Camera,
    pub transform: Transform,
    pub projection: Projection,
}

impl RenderCamera {
    pub fn new(camera: Camera, transform: Transform, projection: Projection) -> Self {
        Self {
            camera,
            transform,
            projection,
        }
    }
}
