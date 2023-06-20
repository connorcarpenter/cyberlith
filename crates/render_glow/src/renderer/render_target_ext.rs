use render_api::components::Viewport;

use crate::renderer::{cmp_render_order, RenderPass};

pub trait RenderTargetExt {
    fn width(&self) -> u32;
    fn height(&self) -> u32;
    fn write(&self, render: impl FnOnce()) -> &Self;

    ///
    /// Render the objects using the given camera and lights into this render target.
    /// Use an empty array for the `lights` argument, if the objects does not require lights to be rendered.
    /// Also, objects outside the camera frustum are not rendered and the objects are rendered in the order given by [cmp_render_order].
    ///
    fn render(&self, render_pass: RenderPass) -> &Self {
        let (camera, lights_holder, mut objects) = render_pass.take();
        let lights = &RenderPass::process_lights(&lights_holder);

        // Forward
        for object in objects.iter_mut() {
            object.finalize();
        }

        // we sort here front->back in order to take advantage of depth-test culling
        objects.sort_by(|a, b| cmp_render_order(&camera, a, b));

        self.write(|| {
            for object in objects {
                object.render(&camera, lights);
            }
        });
        self
    }

    ///
    /// Returns the viewport that encloses the entire target.
    ///
    fn viewport(&self) -> Viewport {
        Viewport::new_at_origin(self.width(), self.height())
    }
}
