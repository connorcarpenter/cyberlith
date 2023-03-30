use crate::asset_impls::AssetImpls;
use crate::core::{ColorTexture, DepthTexture};
use crate::renderer::{Geometry, Light, Material, MaterialType, Object, PostMaterial};
use render_api::base::AxisAlignedBoundingBox;
use render_api::{
    base::{Camera, PbrMaterial, TriMesh},
    Assets, CameraComponent, Handle, Transform,
};

// Render Pass
pub struct RenderPass<'a> {
    pub meshes: &'a AssetImpls<TriMesh, Box<dyn Geometry>>,
    pub materials: &'a AssetImpls<PbrMaterial, Box<dyn Material>>,
    pub camera: &'a Camera,
    pub objects: &'a [RenderObject],
}

impl<'a> RenderPass<'a> {
    pub fn new(
        meshes: &'a AssetImpls<TriMesh, Box<dyn Geometry>>,
        materials: &'a AssetImpls<PbrMaterial, Box<dyn Material>>,
        camera: &'a Camera,
        objects: &'a [RenderObject],
    ) -> Self {
        Self {
            meshes,
            materials,
            camera,
            objects,
        }
    }
}

// Render Object
#[derive(Clone, Copy)]
pub struct RenderObject {
    pub mesh: Handle<TriMesh>,
    pub material: Handle<PbrMaterial>,
    pub transform: Transform,
}

impl RenderObject {
    pub fn new(mesh: Handle<TriMesh>, material: Handle<PbrMaterial>, transform: Transform) -> Self {
        Self {
            mesh,
            material,
            transform,
        }
    }

    pub fn with_assets<'a>(
        &'a self,
        meshes: &'a AssetImpls<TriMesh, Box<dyn Geometry>>,
        materials: &'a AssetImpls<PbrMaterial, Box<dyn Material>>,
    ) -> ActiveRenderObject {
        ActiveRenderObject {
            meshes,
            materials,
            object: self,
        }
    }
}

pub struct ActiveRenderObject<'a> {
    pub meshes: &'a AssetImpls<TriMesh, Box<dyn Geometry>>,
    pub materials: &'a AssetImpls<PbrMaterial, Box<dyn Material>>,
    pub object: &'a RenderObject,
}

impl<'a> Geometry for ActiveRenderObject<'a> {
    fn render_with_material(
        &self,
        material: &dyn Material,
        camera: &Camera,
        lights: &[&dyn Light],
    ) {
        let mesh = self.meshes.get(&self.object.mesh).unwrap();
        mesh.render_with_material(material, camera, lights);
    }

    fn render_with_post_material(
        &self,
        material: &dyn PostMaterial,
        camera: &Camera,
        lights: &[&dyn Light],
        color_texture: Option<ColorTexture>,
        depth_texture: Option<DepthTexture>,
    ) {
        todo!()
    }

    fn aabb(&self) -> AxisAlignedBoundingBox {
        let mesh = self.meshes.get(&self.object.mesh).unwrap();
        mesh.aabb()
    }
}

impl<'a> Object for ActiveRenderObject<'a> {
    fn render(&self, camera: &Camera, lights: &[&dyn Light]) {
        let material = self.materials.get(&self.object.material).unwrap();
        self.render_with_material(material.as_ref(), camera, lights);
    }

    fn material_type(&self) -> MaterialType {
        let material = self.materials.get(&self.object.material).unwrap();
        material.material_type()
    }
}
