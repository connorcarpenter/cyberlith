use crate::{
    core::{ColorTarget, DepthTarget, RenderTarget},
    renderer::{
        cmp_render_order, Geometry, Light, Material, RenderCamera, RenderObject, RenderPass,
    },
};

macro_rules! impl_render_target_extensions_body {
    () => {
        ///
        /// Render the objects using the given camera and lights into this render target.
        /// Use an empty array for the `lights` argument, if the objects does not require lights to be rendered.
        /// Also, objects outside the camera frustum are not rendered and the objects are rendered in the order given by [cmp_render_order].
        ///
        pub fn render(&self, render_pass: RenderPass) -> &Self {
            let (camera, lights_holder, objects) = render_pass.take();
            let lights = &RenderPass::process_lights(&lights_holder);

            let mut forward_objects: Vec<RenderObject> = objects.iter().cloned().collect();

            // Forward

            // we sort here front->back in order to take advantage of depth-test culling
            forward_objects.sort_by(|a, b| cmp_render_order(&camera, a, b));

            self.write(|| {
                for object in forward_objects {
                    object.render(&camera, lights);
                }
            });
            self
        }

        ///
        /// Render the geometries with the given [Material] using the given camera and lights into this render target.
        /// Use an empty array for the `lights` argument, if the material does not require lights to be rendered.
        ///
        pub fn render_with_material(
            &self,
            material: &dyn Material,
            camera: &RenderCamera,
            geometries: impl IntoIterator<Item = impl Geometry>,
            lights: &[&dyn Light],
        ) -> &Self {
            self.write(|| {
                for object in geometries.into_iter() {
                    object.render_with_material(material, camera, lights);
                }
            });
            self
        }
    };
}

macro_rules! impl_render_target_extensions {
    // 2 generic arguments with bounds
    ($name:ident < $a:ident : $ta:tt , $b:ident : $tb:tt >) => {
        impl<$a: $ta, $b: $tb> $name<$a, $b> {
            impl_render_target_extensions_body!();
        }
    };
    // 1 generic argument with bound
    ($name:ident < $a:ident : $ta:tt >) => {
        impl<$a: $ta> $name<$a> {
            impl_render_target_extensions_body!();
        }
    };
    // 1 liftetime argument
    ($name:ident < $lt:lifetime >) => {
        impl<$lt> $name<$lt> {
            impl_render_target_extensions_body!();
        }
    };
    // without any arguments
    ($name:ty) => {
        impl $name {
            impl_render_target_extensions_body!();
        }
    };
}

impl_render_target_extensions!(RenderTarget<'a>);
impl_render_target_extensions!(ColorTarget<'a>);
impl_render_target_extensions!(DepthTarget<'a>);
