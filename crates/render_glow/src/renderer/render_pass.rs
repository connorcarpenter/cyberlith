use render_api::{
    base::{AxisAlignedBoundingBox, Camera, PbrMaterial, TriMesh},
    Handle, Transform,
};

use crate::{
    asset_impls::AssetImpls,
    core::{ColorTexture, DepthTexture},
    renderer::{BaseMesh, Geometry, Light, Material, MaterialType, Mesh, Object, PostMaterial},
};

// Render Pass
pub struct RenderPass<'a> {
    pub camera: &'a Camera,
    pub objects: &'a [RenderObject<'a>],
    pub lights: &'a [&'a dyn Light],
}

impl<'a> RenderPass<'a> {
    pub fn new(
        camera: &'a Camera,
        objects: &'a [RenderObject],
        lights: &'a [&'a dyn Light],
    ) -> Self {
        Self {
            camera,
            objects,
            lights,
        }
    }
}

// // Render Light
// pub struct RenderLight {
//     inner: Box<dyn Light>,
// }
//
// impl RenderLight {
//     pub fn new<T: Light + 'static>(light: T) -> Self {
//         Self { inner: Box::new(light) }
//     }
// }
//
// pub trait AsLights {
//     fn as_lights(&self) -> &[&dyn Light];
// }
//
// impl AsLights for [RenderLight] {
//     fn as_lights(&self) -> &[&dyn Light] {
//         self.iter().map(|light| light.inner.as_ref()).collect::<Vec<_>>().as_slice()
//     }
// }

// Render Object
#[derive(Clone, Copy)]
pub struct RenderObject<'a> {
    pub mesh: &'a BaseMesh,
    pub material: &'a dyn Material,
    pub transform: &'a Transform,
}

impl<'a> RenderObject<'a> {
    pub fn new(mesh: &'a BaseMesh, material: &'a dyn Material, transform: &'a Transform) -> Self {
        Self {
            mesh,
            material,
            transform,
        }
    }
}

impl<'a> Geometry for RenderObject<'a> {
    fn render_with_material(
        &self,
        material: &dyn Material,
        camera: &Camera,
        lights: &[&dyn Light],
    ) {
        let mesh = Mesh::compose(self.mesh, self.transform);
        mesh.render_with_material(material, camera, lights);
    }

    fn render_with_post_material(
        &self,
        _material: &dyn PostMaterial,
        _camera: &Camera,
        _lights: &[&dyn Light],
        _color_texture: Option<ColorTexture>,
        _depth_texture: Option<DepthTexture>,
    ) {
        todo!()
    }

    fn aabb(&self) -> AxisAlignedBoundingBox {
        self.mesh.aabb
    }
}

impl<'a> Object for RenderObject<'a> {
    fn render(&self, camera: &Camera, lights: &[&dyn Light]) {
        self.render_with_material(self.material, camera, lights);
    }

    fn material_type(&self) -> MaterialType {
        self.material.material_type()
    }
}
