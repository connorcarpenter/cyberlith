//!
//! A collection of objects implementing the [Object] trait.
//!
//! Objects can be rendered directly or used in a render call, for example [RenderTarget::render].
//! Use the [Gm] struct to combine any [geometry] and [material] into an [Object].
//!

use render_api::base::Camera;

use crate::renderer::*;

///
/// Represents a 3D object which can be rendered directly or used in a render call, for example [RenderTarget::render].
///
pub trait Object: Geometry {
    ///
    /// Render the object.
    /// Use an empty array for the `lights` argument, if the objects does not require lights to be rendered.
    /// Must be called in the callback given as input to a [RenderTarget], [ColorTarget] or [DepthTarget] write method.
    ///
    fn render(&self, camera: &Camera, lights: &[&dyn Light]);

    ///
    /// Returns the type of material applied to this object.
    ///
    fn material_type(&self) -> MaterialType;
}

impl<T: Object + ?Sized> Object for &T {
    fn render(&self, camera: &Camera, lights: &[&dyn Light]) {
        (*self).render(camera, lights)
    }

    fn material_type(&self) -> MaterialType {
        (*self).material_type()
    }
}

impl<T: Object + ?Sized> Object for &mut T {
    fn render(&self, camera: &Camera, lights: &[&dyn Light]) {
        (**self).render(camera, lights)
    }

    fn material_type(&self) -> MaterialType {
        (**self).material_type()
    }
}

impl<T: Object> Object for Box<T> {
    fn render(&self, camera: &Camera, lights: &[&dyn Light]) {
        self.as_ref().render(camera, lights)
    }

    fn material_type(&self) -> MaterialType {
        self.as_ref().material_type()
    }
}

impl<T: Object> Object for std::sync::Arc<T> {
    fn render(&self, camera: &Camera, lights: &[&dyn Light]) {
        self.as_ref().render(camera, lights)
    }

    fn material_type(&self) -> MaterialType {
        self.as_ref().material_type()
    }
}

impl<T: Object> Object for std::sync::RwLock<T> {
    fn render(&self, camera: &Camera, lights: &[&dyn Light]) {
        self.read().unwrap().render(camera, lights)
    }

    fn material_type(&self) -> MaterialType {
        self.read().unwrap().material_type()
    }
}
