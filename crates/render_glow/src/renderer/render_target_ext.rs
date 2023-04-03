use render_api::base::{Camera, Interpolation, Viewport, Wrapping};

use crate::{
    core::{
        ClearState, ColorTarget, ColorTargetMultisample, ColorTexture, DepthTarget,
        DepthTargetMultisample, DepthTexture, DepthTexture2D, DepthTextureDataType, RenderTarget,
        RenderTargetMultisample, ScissorBox, Texture2DArray, TextureDataType,
    },
    renderer::{
        cmp_render_order, DeferredPhysicalMaterial, Geometry, Light, Material, MaterialType,
        Object, PostMaterial, RenderObject, RenderPass,
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
            self.render_partially(self.scissor_box(), render_pass)
        }

        ///
        /// Render the objects using the given camera and lights into the part of this render target defined by the scissor box.
        /// Use an empty array for the `lights` argument, if the objects does not require lights to be rendered.
        /// Also, objects outside the camera frustum are not rendered and the objects are rendered in the order given by [cmp_render_order].
        ///
        pub fn render_partially(&self, scissor_box: ScissorBox, render_pass: RenderPass) -> &Self {
            let RenderPass {
                camera,
                objects,
                lights,
            } = render_pass;

            let (mut deferred_objects, mut forward_objects): (
                Vec<RenderObject>,
                Vec<RenderObject>,
            ) = objects
                .iter()
                .filter(|o| camera.in_frustum(&o.aabb()))
                .partition(|o| o.material_type() == MaterialType::Deferred);

            // Deferred
            if deferred_objects.len() > 0 {
                // Geometry pass
                let mut geometry_pass_camera = camera.clone();
                let viewport =
                    Viewport::new_at_origin(camera.viewport().width, camera.viewport().height);
                geometry_pass_camera.set_viewport(viewport);
                deferred_objects.sort_by(|a, b| cmp_render_order(&geometry_pass_camera, a, b));
                let mut geometry_pass_texture = Texture2DArray::new_empty::<[u8; 4]>(
                    viewport.width,
                    viewport.height,
                    3,
                    Interpolation::Nearest,
                    Interpolation::Nearest,
                    None,
                    Wrapping::ClampToEdge,
                    Wrapping::ClampToEdge,
                );
                let mut geometry_pass_depth_texture = DepthTexture2D::new::<f32>(
                    viewport.width,
                    viewport.height,
                    Wrapping::ClampToEdge,
                    Wrapping::ClampToEdge,
                );
                let gbuffer_layers = [0, 1, 2];
                RenderTarget::new(
                    geometry_pass_texture.as_color_target(&gbuffer_layers, None),
                    geometry_pass_depth_texture.as_depth_target(),
                )
                .clear(ClearState::default())
                .write(|| {
                    for object in deferred_objects {
                        object.render(&geometry_pass_camera, lights);
                    }
                });

                // Lighting pass
                self.write_partially(scissor_box, || {
                    DeferredPhysicalMaterial::lighting_pass(
                        camera,
                        ColorTexture::Array {
                            texture: &geometry_pass_texture,
                            layers: &gbuffer_layers,
                        },
                        DepthTexture::Single(&geometry_pass_depth_texture),
                        lights,
                    )
                });
            }

            // Forward
            forward_objects.sort_by(|a, b| cmp_render_order(camera, a, b));
            self.write_partially(scissor_box, || {
                for object in forward_objects {
                    object.render(camera, lights);
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
            camera: &Camera,
            geometries: impl IntoIterator<Item = impl Geometry>,
            lights: &[&dyn Light],
        ) -> &Self {
            self.render_partially_with_material(
                self.scissor_box(),
                material,
                camera,
                geometries,
                lights,
            )
        }

        ///
        /// Render the geometries with the given [Material] using the given camera and lights into the part of this render target defined by the scissor box.
        /// Use an empty array for the `lights` argument, if the material does not require lights to be rendered.
        ///
        pub fn render_partially_with_material(
            &self,
            scissor_box: ScissorBox,
            material: &dyn Material,
            camera: &Camera,
            geometries: impl IntoIterator<Item = impl Geometry>,
            lights: &[&dyn Light],
        ) -> &Self {
            self.write_partially(scissor_box, || {
                for object in geometries
                    .into_iter()
                    .filter(|o| camera.in_frustum(&o.aabb()))
                {
                    object.render_with_material(material, camera, lights);
                }
            });
            self
        }

        ///
        /// Render the geometries with the given [PostMaterial] using the given camera and lights into this render target.
        /// Use an empty array for the `lights` argument, if the material does not require lights to be rendered.
        ///
        pub fn render_with_post_material(
            &self,
            material: &dyn PostMaterial,
            camera: &Camera,
            geometries: impl IntoIterator<Item = impl Geometry>,
            lights: &[&dyn Light],
            color_texture: Option<ColorTexture>,
            depth_texture: Option<DepthTexture>,
        ) -> &Self {
            self.render_partially_with_post_material(
                self.scissor_box(),
                material,
                camera,
                geometries,
                lights,
                color_texture,
                depth_texture,
            )
        }

        ///
        /// Render the geometries with the given [PostMaterial] using the given camera and lights into the part of this render target defined by the scissor box.
        /// Use an empty array for the `lights` argument, if the material does not require lights to be rendered.
        ///
        pub fn render_partially_with_post_material(
            &self,
            scissor_box: ScissorBox,
            material: &dyn PostMaterial,
            camera: &Camera,
            geometries: impl IntoIterator<Item = impl Geometry>,
            lights: &[&dyn Light],
            color_texture: Option<ColorTexture>,
            depth_texture: Option<DepthTexture>,
        ) -> &Self {
            self.write_partially(scissor_box, || {
                for object in geometries
                    .into_iter()
                    .filter(|o| camera.in_frustum(&o.aabb()))
                {
                    object.render_with_post_material(
                        material,
                        camera,
                        lights,
                        color_texture,
                        depth_texture,
                    );
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
impl_render_target_extensions!(RenderTargetMultisample<C: TextureDataType, D: DepthTextureDataType>);
impl_render_target_extensions!(ColorTargetMultisample<C: TextureDataType>);
impl_render_target_extensions!(DepthTargetMultisample<D: DepthTextureDataType>);
